use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::WidgetRef};

use crate::{
    app::App,
    error_display::{self, AppError, Notification},
    keys::AppKey,
    pane::ErrorPane,
};

impl App {
    pub fn show_notification(&mut self, error: AppError) {
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
}
