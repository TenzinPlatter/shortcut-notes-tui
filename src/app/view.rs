#![cfg(any())]

use ratatui::{
    buffer::Buffer,
    layout::{Direction, Rect},
    widgets::WidgetRef,
};

use crate::{
    app::App,
    pane::ParagraphPane,
    view::{View, ViewBuilder},
};

impl App {
    pub fn get_loading_view_iteration() -> View {
        let loading_pane = ParagraphPane::loading("iteration");
        ViewBuilder::default()
            .add_non_selectable(loading_pane)
            .direction(Direction::Vertical)
            .build()
    }

    pub fn get_loading_view_stories() -> View {
        let loading_pane = ParagraphPane::loading("stories");
        ViewBuilder::default()
            .add_non_selectable(loading_pane)
            .direction(Direction::Vertical)
            .build()
    }
}

impl WidgetRef for &App {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.view.render_ref(area, buf);
    }
}
