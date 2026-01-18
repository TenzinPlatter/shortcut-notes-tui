use core::error;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Direction, Rect},
    widgets::WidgetRef,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    api::{ApiClient, epic::Epic},
    error_display::{self, AppError, ErrorExt, ErrorHandler, ErrorSeverity, Notification},
    get_api_key, get_user_id,
    keys::{AppKey, KeyHandler},
    pane::{ErrorPane, ParagraphPane},
    view::{View, ViewBuilder},
};

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

    fn get_loading_view() -> View {
        let loading_pane = ParagraphPane::loading();
        ViewBuilder::default()
            .add_non_selectable(loading_pane)
            .direction(Direction::Vertical)
            .build()
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
                self.view = Self::create_epics_view(epics);
            }
            AppEvent::EpicsFailed(error) => {
                self.show_notification(error);
            }
        }
    }

    fn create_epics_view(epics: Vec<Epic>) -> View {
        let panes: Vec<_> = epics.iter().map(ParagraphPane::epic).collect();

        ViewBuilder::default().add_panes(panes).build()
    }

    pub fn show_notification(&mut self, error: error_display::AppError) {
        let (pane, _) = self.error_handler.handle(&error);
        self.notification = Some(Notification::new(pane));
    }

    pub fn show_blocking_error(
        terminal: &mut DefaultTerminal,
        mut error_pane: ErrorPane,
    ) -> error_display::Result<()> {
        let view = error_pane.as_view();

        // Draw once
        terminal.draw(|frame| {
            let area = frame.area();
            view.render_ref(area, frame.buffer_mut())
        })?;

        // Wait for any key
        loop {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                if key.code == AppKey::ShowErrorDetails.into() {
                    error_pane.toggle_details();
                    let view = error_pane.as_view();
                    terminal.draw(|frame| {
                        let area = frame.area();
                        view.render_ref(area, frame.buffer_mut())
                    })?;
                } else {
                    // break on any other key
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn run(terminal: &mut DefaultTerminal) -> error_display::Result<()> {
        match Self::init().await {
            Ok(mut app) => app.main_loop(terminal).await,
            Err(e) => {
                let error_handler = ErrorHandler;
                let (error_pane, severity) = error_handler.handle(&e);

                match severity {
                    ErrorSeverity::Blocking => {
                        // Don't propagate errors from show_blocking_error - just try our best
                        let _ = Self::show_blocking_error(terminal, error_pane);
                        Ok(()) // Exit gracefully after showing error
                    }
                    ErrorSeverity::Notification => {
                        // This shouldn't happen during init, but handle anyway
                        Err(e) // Propagate the error
                    }
                }
            }
        }
    }
}

impl WidgetRef for &App {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.view.render_ref(area, buf);
    }
}
