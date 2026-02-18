use std::fs::read_to_string;
use std::io::Write;

use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Clear, Paragraph, StatefulWidget, WidgetRef};
use ratatui::{DefaultTerminal, Frame};
use tempfile::NamedTempFile;
use tokio::sync::mpsc;

use crate::app::pane::action_menu::ActionMenu;
use crate::error::{ERROR_NOTIFICATION_MAX_HEIGHT, ErrorInfo};
use crate::view::description_modal::{DescriptionModal, centered_rect};
use crate::view::{navbar::NavBar, notes_list::NotesListView, story_list::StoryListView};
use crate::worktree::{create_worktree, get_repo_list, select_repo_with_fzf};
use crate::{api::ApiClient, app::model::ViewType, config::Config};

pub mod cmd;
pub mod init;
pub mod model;
pub mod msg;
pub mod pane;
pub mod update;

pub struct App {
    pub model: model::Model,
    pub exit: bool,
    pub receiver: mpsc::UnboundedReceiver<msg::Msg>,
    pub sender: mpsc::UnboundedSender<msg::Msg>,
    pub api_client: ApiClient,
    pub config: Config,
}

impl App {
    pub async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if let Some(msg) = self.poll_for_message().await? {
                let commands = self.update(msg);

                for cmd in commands {
                    match cmd {
                        cmd::Cmd::OpenNote { .. }
                        | cmd::Cmd::OpenIterationNote { .. }
                        | cmd::Cmd::EditStoryContent { .. }
                        | cmd::Cmd::CreateGitWorktree { .. }
                        | cmd::Cmd::OpenDailyNote { .. } => {
                            self.handle_suspended_cmd(cmd, terminal).await?;
                        }
                        _ => {
                            cmd::execute(
                                cmd,
                                self.sender.clone(),
                                &mut self.model,
                                &self.api_client,
                            )
                            .await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_suspended_cmd(
        &mut self,
        cmd: cmd::Cmd,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        match cmd {
            cmd::Cmd::OpenNote {
                story_id,
                story_name,
                story_app_url,
                iteration_app_url,
            } => {
                with_suspended_tui(terminal, || {
                    cmd::open_note_in_editor(
                        story_id,
                        story_name,
                        story_app_url,
                        iteration_app_url,
                        &self.model.config,
                    )
                })?;
                self.sender.send(msg::Msg::NoteOpened).ok();
            }

            cmd::Cmd::OpenIterationNote {
                iteration_id,
                iteration_name,
                iteration_app_url,
            } => {
                with_suspended_tui(terminal, || {
                    cmd::open_iteration_note_in_editor(
                        iteration_id,
                        iteration_name,
                        iteration_app_url,
                        &self.model.config,
                    )
                })?;
                self.sender.send(msg::Msg::NoteOpened).ok();
            }

            cmd::Cmd::EditStoryContent {
                story_id,
                description,
            } => {
                let config_editor = self.model.config.editor.clone();
                let edited = with_suspended_tui(terminal, || {
                    let mut tempfile = NamedTempFile::new()?;
                    tempfile.write_all(description.as_bytes())?;
                    let tmp_path = tempfile.path().to_path_buf();

                    std::process::Command::new(&config_editor)
                        .arg(&tmp_path)
                        .status()?;

                    let contents = read_to_string(&tmp_path)?;
                    Ok(contents)
                })?;

                if edited != description {
                    self.api_client
                        .update_story_description(story_id, edited)
                        .await?;
                }
            }

            cmd::Cmd::CreateGitWorktree { branch_name } => {
                let repos = get_repo_list(&self.model.config).await?;
                let chosen = match with_suspended_tui(terminal, || select_repo_with_fzf(&repos)) {
                    Ok(repo) => repo,
                    Err(e) => {
                        self.model
                            .ui
                            .errors
                            .push(ErrorInfo::new("Failed to get repo for worktree", e));
                        return Ok(());
                    }
                };

                let path = self.model.config.repositories_directory.join(chosen);
                create_worktree(&path, &branch_name).await?;
            }

            cmd::Cmd::OpenDailyNote { path } => {
                with_suspended_tui(terminal, || {
                    cmd::open_daily_note_with_frontmatter(&self.model.config, &path)
                })?;
                self.sender.send(msg::Msg::NoteOpened).ok();
            }

            _ => unreachable!("Non-suspending command passed to handle_suspended_cmd"),
        }

        Ok(())
    }

    async fn poll_for_message(&mut self) -> Result<Option<msg::Msg>> {
        use crossterm::event::{self, Event, KeyEventKind};
        use std::time::Duration;

        tokio::select! {
            terminal_event = tokio::task::spawn_blocking(|| {
                if event::poll(Duration::from_millis(100))? {
                    event::read()
                } else {
                    Ok(Event::Resize(0, 0))
                }
            }) => {
                match terminal_event?? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        Ok(Some(msg::Msg::KeyPressed(key)))
                    }
                    _ => Ok(None)
                }
            }

            Some(msg) = self.receiver.recv() => {
                Ok(Some(msg))
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        // Advance spinner animation
        self.model.ui.throbber_state.calc_next();
        let tick = self.model.ui.throbber_state.index().unsigned_abs() as usize;

        // Split screen: navbar at top, main view below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Navbar: border + content + border
                Constraint::Min(0),    // Main view: everything else
            ])
            .split(frame.area());

        // Render navbar
        let has_stories = !self.model.data.stories.is_empty();
        let navbar = NavBar::new(
            self.model.ui.active_view,
            self.model.ui.loading,
            has_stories,
            tick,
        );

        navbar.render_ref(chunks[0], frame.buffer_mut());

        // Render main view based on active_view
        match self.model.ui.active_view {
            ViewType::Stories => {
                let story_list_view = StoryListView::new(
                    &self.model.data.stories,
                    self.model.data.current_iterations.as_deref(),
                    &self.model.ui.story_list,
                    self.model.data.active_story.as_ref(),
                    true, // Always focused (single view)
                    self.model.ui.loading,
                    tick,
                );

                story_list_view.render_ref(chunks[1], frame.buffer_mut());
            }

            ViewType::Iterations => {
                // let iteration_view = IterationListView::new();
                let placeholder = Paragraph::new("Coming soon...").block(Block::bordered());
                frame.render_widget(placeholder, chunks[1]);
            }

            ViewType::Notes => {
                let notes_view = NotesListView::new(&self.model.ui.notes_list);
                notes_view.render_ref(chunks[1], frame.buffer_mut());
            }

            ViewType::Epics | ViewType::Search => {
                // Placeholder for future views
                let placeholder = Paragraph::new("Coming soon...").block(Block::bordered());
                frame.render_widget(placeholder, chunks[1]);
            }
        }

        if self.model.ui.action_menu.is_showing {
            let (width, height) = ActionMenu::window_dimensions();
            let (width, height) = (width as u16, height as u16);
            let x = (frame.area().width - width) / 2;
            let y = (frame.area().height - height) / 2;

            let rect = Rect::new(x, y, width, height);
            ActionMenu.render(
                rect,
                frame.buffer_mut(),
                &mut self.model.ui.action_menu.list_state,
            );
        }

        // Render description modal (highest priority overlay before errors)
        if self.model.ui.description_modal.is_showing
            && let Some(story) = &self.model.ui.description_modal.story
        {
            let area = centered_rect(80, 80, frame.area());
            frame.render_widget(Clear, area);

            let modal = DescriptionModal::new(story);
            frame.render_stateful_widget(
                modal,
                area,
                &mut self.model.ui.description_modal.scroll_view_state,
            );
        }

        self.draw_error(frame);
    }

    fn draw_error(&self, frame: &mut Frame) {
        let mut used_height = 0;
        for error in self.model.ui.errors.iter().filter(|e| !e.is_expired()) {
            let width = error.get_required_width();
            let height = error.get_required_height(width);
            let area = frame.area();
            let area = Rect::new(
                area.width - width,
                used_height,
                width,
                height.min(ERROR_NOTIFICATION_MAX_HEIGHT),
            );

            used_height += height;

            // clear terminal area, stops characters behind empty space from being visible
            frame.render_widget(Clear, area);

            error.render_ref(area, frame.buffer_mut());
        }
    }
}

fn with_suspended_tui<F, T>(terminal: &mut DefaultTerminal, f: F) -> anyhow::Result<T>
where
    F: FnOnce() -> anyhow::Result<T>,
{
    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    let result = f();
    std::io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    result
}
