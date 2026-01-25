#![cfg(any())]

use crate::{
    api::epic::Epic,
    pane::ParagraphPane,
    view::{View, ViewBuilder},
};

// TODO: move to list
pub fn create_epics_view(epics: Vec<Epic>) -> View {
    let panes: Vec<_> = epics.iter().map(ParagraphPane::epic).collect();
    ViewBuilder::default().add_panes(panes).build()
}
