use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph, Widget, WidgetRef},
};

use crate::{
    api::epic::EpicSlim,
    app::{model::EpicListState, pane::epic_list::filter_items},
    custom_list::LinearList,
};

pub struct EpicListView<'a> {
    pub epics: &'a [EpicSlim],
    pub state: &'a EpicListState,
}

impl<'a> EpicListView<'a> {
    pub fn new(epics: &'a [EpicSlim], state: &'a EpicListState) -> EpicListView<'a> {
        EpicListView { epics, state }
    }
}

impl WidgetRef for EpicListView<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

        let query = &self.state.search_query;
        let display = if query.is_empty() && !self.state.search_active {
            "/ to search".to_string()
        } else {
            format!("{}_", query)
        };
        let bar_style = if self.state.search_active {
            Style::new().yellow()
        } else {
            Style::new().dark_gray()
        };
        Paragraph::new(display)
            .block(Block::bordered().title(" Search "))
            .style(bar_style)
            .render(chunks[0], buf);

        let visible: Vec<EpicSlim> = filter_items(self.epics, query)
            .into_iter()
            .cloned()
            .collect();
        let empty_msg = if query.is_empty() {
            "No epics."
        } else {
            "No results."
        };
        LinearList::new(&visible, self.state.selected_id, empty_msg).render_ref(chunks[1], buf);
    }
}
