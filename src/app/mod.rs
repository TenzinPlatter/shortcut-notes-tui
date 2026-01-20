use anyhow::Context;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, widgets::WidgetRef};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    api::{
        ApiClient,
        epic::{Epic, view::create_epics_view},
        story::{Story, view::create_stories_view},
    },
    get_api_key, get_user_id,
    keys::{AppKey, KeyHandler},
    view::View,
};

pub mod view;

/// Events sent from background tasks to the main app
pub enum AppEvent {
    UnexpectedError(anyhow::Error),
    EpicsLoaded(Vec<Epic>),
    StoriesLoaded(Vec<Story>),
    IterationLoaded,
}

pub struct App {
    pub view: View,
    exit: bool,
    api_client: ApiClient,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    epics: Option<Vec<Epic>>,
    stories: Option<Vec<Story>>,
}

impl App {
    pub async fn init() -> anyhow::Result<Self> {
        let api_client = {
            let api_key = get_api_key().await?;
            let user_id = get_user_id()
                .await?
                .parse::<Uuid>()
                .context("Got invalid user id")?;
            ApiClient::new(api_key, user_id)
        };

        // Create channel for background tasks to communicate with main app
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Start with a loading view
        let view = Self::get_loading_view_iteration();

        // Spawn background task to fetch epics
        let api_client_clone = api_client.clone();
        tokio::spawn(async move {
            let iteration = match api_client_clone.get_current_iteration().await {
                Ok(iteration) => {
                    let _ = event_tx.send(AppEvent::IterationLoaded);
                    iteration
                },
                Err(e) => {
                    let _ = event_tx.send(AppEvent::UnexpectedError(e));
                    return;
                }
            };

            match api_client_clone.get_owned_iteration_stories(&iteration).await {
                Ok(stories) => {
                    let _ = event_tx.send(AppEvent::StoriesLoaded(stories));
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::UnexpectedError(e));
                }
            }
        });

        Ok(Self {
            view,
            exit: false,
            api_client,
            event_rx,
            epics: None,
            stories: None,
        })
    }

    pub async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.view.render_ref(frame.area(), frame.buffer_mut());
    }

    async fn handle_events(&mut self) -> anyhow::Result<()> {
        tokio::select! {
            // Poll for terminal events with a timeout to avoid blocking forever
            terminal_event = tokio::task::spawn_blocking(|| {
                if event::poll(std::time::Duration::from_millis(100))? {
                    event::read()
                } else {
                    Ok(Event::Resize(0, 0)) // Dummy event to indicate no real event
                }
            }) => {
                match terminal_event?? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        // Dismiss notification on any key
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
                self.handle_app_event(app_event)?;
            }
        }

        Ok(())
    }

    fn handle_app_event(&mut self, event: AppEvent) -> anyhow::Result<()> {
        match event {
            AppEvent::UnexpectedError(e) => {
                return Err(e);
            }
            AppEvent::EpicsLoaded(epics) => {
                self.view = create_epics_view(epics);
            }
            AppEvent::StoriesLoaded(stories) => {
                self.view = create_stories_view(stories);
            }
            AppEvent::IterationLoaded => {
                self.view = App::get_loading_view();
            },
        }

        Ok(())
    }
}
