use std::fs::read_to_string;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Clear, Paragraph, StatefulWidget, WidgetRef, Widget};
use ratatui::{DefaultTerminal, Frame};
use tempfile::NamedTempFile;
use tokio::sync::mpsc;

use crate::app::pane::action_menu::ActionMenu;
use crate::error::{ERROR_NOTIFICATION_MAX_HEIGHT, ErrorInfo};
use crate::view::add_todo_modal::AddTodoModal;
use crate::view::create_note_modal::CreateNoteModal;
use crate::view::description_modal::{DescriptionModal, centered_rect};
use crate::view::keybinds_panel::KeybindsPanel;
use crate::view::todos_list::TodosListView;
use crate::view::{EpicListView, IterationListView};
use crate::view::{navbar::NavBar, notes_list::NotesListView, story_list::StoryListView};
use crate::worktree::{create_worktree, get_repo_list, select_repo_with_fzf};
use crate::{api::{ApiClient, story::Story}, app::model::ViewType};
use arc_core::Config;

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
    /// When set, the key-reader thread stops consuming terminal input so an
    /// external program (editor, fzf) launched via `with_suspended_tui` can own
    /// the terminal.
    pub key_reader_paused: Arc<AtomicBool>,
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
                        | cmd::Cmd::OpenEpicNote { .. }
                        | cmd::Cmd::EditStoryContent { .. }
                        | cmd::Cmd::CreateGitWorktree { .. }
                        | cmd::Cmd::OpenDailyNote { .. }
                        | cmd::Cmd::OpenScratchNote { .. }
                        | cmd::Cmd::OpenTmuxSession { .. } => {
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
                with_suspended_tui(&self.key_reader_paused, terminal,|| {
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
                with_suspended_tui(&self.key_reader_paused, terminal,|| {
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
                let edited = with_suspended_tui(&self.key_reader_paused, terminal,|| {
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
                let chosen = match with_suspended_tui(&self.key_reader_paused, terminal,|| select_repo_with_fzf(&repos)) {
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

            cmd::Cmd::OpenEpicNote {
                epic_id,
                epic_name,
                epic_app_url,
            } => {
                with_suspended_tui(&self.key_reader_paused, terminal,|| {
                    cmd::open_epic_note_in_editor(
                        epic_id,
                        epic_name,
                        epic_app_url,
                        &self.model.config,
                    )
                })?;
                self.sender.send(msg::Msg::NoteOpened).ok();
            }

            cmd::Cmd::OpenDailyNote { path } => {
                with_suspended_tui(&self.key_reader_paused, terminal,|| {
                    cmd::open_daily_note_with_frontmatter(&self.model.config, &path)
                })?;
                self.sender.send(msg::Msg::NoteOpened).ok();
            }

            cmd::Cmd::OpenScratchNote { path, name } => {
                with_suspended_tui(&self.key_reader_paused, terminal,|| {
                    cmd::open_scratch_note_in_editor(&name, &path, &self.model.config)
                })?;
                self.sender.send(msg::Msg::NoteOpened).ok();
            }

            cmd::Cmd::OpenTmuxSession { story_name } => {
                let session_name = Story::tmux_session_name(&story_name);
                let mux = self.model.config.mux.clone();
                with_suspended_tui(&self.key_reader_paused, terminal,|| {
                    cmd::open_mux_session_sync(&session_name, &mux)
                })?;
            }

            _ => unreachable!("Non-suspending command passed to handle_suspended_cmd"),
        }

        Ok(())
    }

    async fn poll_for_message(&mut self) -> Result<Option<msg::Msg>> {
        // Key events and async messages all arrive on the one channel, in order.
        // Terminal input is read on a dedicated thread (see `spawn_key_reader`),
        // so nothing is lost to select-cancellation here.
        Ok(self.receiver.recv().await)
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
                    self.model.ui.loading,
                    tick,
                );

                story_list_view.render_ref(chunks[1], frame.buffer_mut());
            }

            ViewType::Notes => {
                let notes_view = NotesListView::new(&self.model.ui.notes_list);
                notes_view.render_ref(chunks[1], frame.buffer_mut());
            }

            ViewType::Iterations => {
                let active = self.model.data.current_iterations.as_deref().unwrap_or(&[]);
                let iteration_list = IterationListView::new(
                    active,
                    &self.model.data.iterations,
                    &self.model.ui.iteration_list,
                );
                iteration_list.render_ref(chunks[1], frame.buffer_mut());
            }

            ViewType::Epics => {
                let epic_list =
                    EpicListView::new(&self.model.data.epics, &self.model.ui.epic_list);
                epic_list.render_ref(chunks[1], frame.buffer_mut());
            }

            ViewType::Todos => {
                let todos_view = TodosListView::new(
                    &self.model.data.todos,
                    &self.model.ui.todos_list,
                );
                todos_view.render_ref(chunks[1], frame.buffer_mut());
            }

            ViewType::Search => {
                // Placeholder for future views
                let placeholder = Paragraph::new("Coming soon...").block(Block::bordered());
                placeholder.render(chunks[1], frame.buffer_mut());
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
            Clear.render(area, frame.buffer_mut());

            let modal = DescriptionModal::new(story);
            modal.render(
                area,
                frame.buffer_mut(),
                &mut self.model.ui.description_modal.scroll_view_state,
            );
        }

        // Render create note modal on top when showing
        if self.model.ui.create_note_modal.is_showing {
            let area = frame.area();
            Clear.render(centered_rect(50, 30, area), frame.buffer_mut());
            let modal = CreateNoteModal::new(&self.model.ui.create_note_modal);
            modal.render_ref(area, frame.buffer_mut());
        }

        // Render add todo modal on top when showing
        if self.model.ui.add_todo_modal.is_showing {
            let area = frame.area();
            Clear.render(centered_rect(50, 30, area), frame.buffer_mut());
            let modal = AddTodoModal::new(&self.model.ui.add_todo_modal);
            modal.render_ref(area, frame.buffer_mut());
        }

        // Render keybinds panel (above description modal, below errors)
        if self.model.ui.show_keybinds_panel {
            KeybindsPanel.render(frame.area(), frame.buffer_mut());
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
            Clear.render(area, frame.buffer_mut());

            error.render_ref(area, frame.buffer_mut());
        }
    }
}

fn with_suspended_tui<F, T>(
    key_reader_paused: &AtomicBool,
    terminal: &mut DefaultTerminal,
    f: F,
) -> anyhow::Result<T>
where
    F: FnOnce() -> anyhow::Result<T>,
{
    // Stop the key-reader thread from consuming input while the external program
    // owns the terminal.
    key_reader_paused.store(true, Ordering::SeqCst);
    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    let result = f();
    std::io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    key_reader_paused.store(false, Ordering::SeqCst);
    result
}

/// Spawn a dedicated OS thread that reads terminal input and forwards key-press
/// events into the unified `Msg` channel. Reading continuously on its own thread
/// means crossterm's input buffer is always drained, so fast keystrokes are never
/// dropped (the previous async-`select!` approach could cancel a pending read).
fn spawn_key_reader(sender: mpsc::UnboundedSender<msg::Msg>, paused: Arc<AtomicBool>) {
    use crossterm::event::{self, Event, KeyEventKind};
    use std::time::Duration;

    std::thread::spawn(move || {
        loop {
            if paused.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }

            // Short poll timeout so the pause flag is observed promptly instead
            // of blocking indefinitely in `read()`.
            match event::poll(Duration::from_millis(100)) {
                Ok(true) => match event::read() {
                    Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                        if sender.send(msg::Msg::KeyPressed(key)).is_err() {
                            break; // receiver dropped — app is exiting
                        }
                    }
                    Ok(_) => {}
                    Err(_) => break,
                },
                Ok(false) => {}
                Err(_) => break,
            }
        }
    });
}
