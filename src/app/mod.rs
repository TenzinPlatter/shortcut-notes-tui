use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Direction, Rect},
    widgets::WidgetRef,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    api::{
        ApiClient,
        epic::{Epic, view::create_epics_view},
    },
    error_display::{self, AppError, ErrorExt, ErrorHandler, ErrorSeverity, Notification},
    get_api_key, get_user_id,
    keys::{AppKey, KeyHandler},
    pane::{ErrorPane, ParagraphPane},
    view::{View, ViewBuilder},
};

pub mod error_handling;
pub mod view;

/// Events sent from background tasks to the main app
pub enum AppEvent {
    EpicsLoaded(Vec<Epic>),
    EpicsFailed(error_display::AppError),
}

pub struct App {
    pub view: View,
    exit: bool,
    api_client: ApiClient,
    notification: Option<Notification>,
    error_handler: ErrorHandler,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl App {
    pub async fn init() -> error_display::Result<Self> {
        let api_key = get_api_key().await?;
        let user_id = get_user_id()
            .await?
            .parse::<Uuid>()
            .blocking()
            .with_message("Got invalid user id")?;
        let api_client = ApiClient::new(api_key, user_id);

        // Create channel for background tasks to communicate with main app
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Start with a loading view
        let view = Self::get_loading_view();

        // Spawn background task to fetch epics
        let api_client_clone = api_client.clone();
        tokio::spawn(async move {
            match api_client_clone.get_owned_epics().await {
                Ok(epics) => {
                    let _ = event_tx.send(AppEvent::EpicsLoaded(epics));
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::EpicsFailed(e));
                }
            }
        });

        Ok(Self {
            view,
            exit: false,
            api_client,
            notification: None,
            error_handler: ErrorHandler,
            event_rx,
        })
    }

    pub async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> error_display::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?; // blocks until an event occurs, thus only draw on change
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        // Check if notification expired
        if let Some(notif) = &self.notification
            && notif.is_expired()
        {
            self.notification = None;
        }

        // Draw main view
        let area = frame.area();
        self.view.render_ref(area, frame.buffer_mut());

        // Render notification overlay if present
        if let Some(notif) = &self.notification {
            let notif_area = Rect {
                x: area.width.saturating_sub(40),
                y: area.height.saturating_sub(5),
                width: 38.min(area.width),
                height: 4.min(area.height),
            };
            notif.pane.render_ref(notif_area, frame.buffer_mut());
        }
    }

    async fn handle_events(&mut self) -> error_display::Result<()> {
        tokio::select! {
            terminal_event = tokio::task::spawn_blocking(event::read) => {
                match terminal_event?? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        // Dismiss notification on any key
                        if self.notification.is_some() {
                            self.notification = None;
                            return Ok(());
                        }

                        if key_event.code == AppKey::Quit.into() {
                            self.exit = true;
                        } else {
                            self.view.handle_key_event(key_event);
                        }
                    }
                    _ => {}
                }
            }

            // Handle app events from background tasks
            Some(app_event) = self.event_rx.recv() => {
                self.handle_app_event(app_event);
            }
        }

        Ok(())
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::EpicsLoaded(epics) => {
                self.view = create_epics_view(epics);
            }
            AppEvent::EpicsFailed(error) => {
                self.show_notification(error);
            }
        }
    }
}
