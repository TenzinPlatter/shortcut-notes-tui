use crate::{
    api::story::Story,
    pane::ParagraphPane,
    view::{View, ViewBuilder},
};

// TODO: move to list
pub fn create_stories_view(stories: Vec<Story>) -> View {
    let panes: Vec<_> = stories.iter().map(ParagraphPane::story).collect();
    ViewBuilder::default().add_panes(panes).build()
}
