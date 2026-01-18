use crate::api::epic::Epic;

pub struct EpicPane {
    epics: Vec<Epic>,
}

impl EpicPane {
    pub fn new(epics: Vec<Epic>) -> Self {
        Self { epics }
    }
}
