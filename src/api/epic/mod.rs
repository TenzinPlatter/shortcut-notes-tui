use serde::Deserialize;
use uuid::Uuid;

use crate::api::ApiClient;

#[derive(Deserialize)]
pub struct EpicSlim {
    pub id: i32,
    pub owner_ids: Vec<Uuid>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Epic {
    pub id: i32,
    pub app_url: String,
    pub completed: bool,
    pub description: String,
    pub name: String,
    pub owner_ids: Vec<Uuid>,
    pub started: bool,
}

impl ApiClient {
    pub async fn get_owned_epics(&self) -> anyhow::Result<Vec<Epic>> {
        let body = serde_json::json!({
            "includes_description": false
        });

        let response = self.get_with_body("epics", &body).await?;
        let epics_slim = response.json::<Vec<EpicSlim>>().await?;

        let owned_slim = epics_slim
            .into_iter()
            .filter(|epic| epic.owner_ids.contains(&self.user_id))
            .collect::<Vec<_>>();

        let mut epics = Vec::new();
        for epic in owned_slim.iter().take(2) {
            let response = self.get(&format!("epics/{}", epic.id)).await?;
            let epic = response.json::<Epic>().await?;
            epics.push(epic);
        }

        Ok(epics)
    }
}
