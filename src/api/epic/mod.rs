use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{api::ApiClient, custom_list::LinearListItem};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EpicSlim {
    pub id: i32,
    pub name: String,
    pub app_url: String,
    pub owner_ids: Vec<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

impl LinearListItem for EpicSlim {
    fn id(&self) -> i32 { self.id }
    fn label(&self) -> &str { &self.name }
}

impl ApiClient {
    pub async fn get_all_epics_slim(&self, include_description: bool) -> anyhow::Result<Vec<EpicSlim>> {
        let body = serde_json::json!({
            "includes_description": include_description
        });

        let response = self.get_with_body("epics", &body).await?;
        let epics_slim = response.json::<Vec<EpicSlim>>().await?;

        Ok(epics_slim)
    }

    pub async fn get_owned_epics(&self) -> anyhow::Result<Vec<Epic>> {
        let epics_slim = self.get_all_epics_slim(false).await?;

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
