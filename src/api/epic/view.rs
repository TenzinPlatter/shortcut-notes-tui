use crate::{
    api::{epic::Epic, ApiClient},
    pane::ParagraphPane,
    view::{View, ViewBuilder},
};

impl ApiClient {
    pub async fn get_epics_view(&self) -> crate::error_display::Result<View> {
        let epics = self.get_owned_epics().await?;
        let panes: Vec<_> = epics
            .into_iter()
            .map(|epic| ParagraphPane::epic(&epic))
            .collect();

        Ok(ViewBuilder::default().add_panes(panes).build())
    }
}

pub fn create_epics_view(epics: Vec<Epic>) -> View {
    let panes: Vec<_> = epics.iter().map(ParagraphPane::epic).collect();
    ViewBuilder::default().add_panes(panes).build()
}
