use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::get_full_path;

#[derive(Deserialize)]
pub struct Member {
    id: Uuid,
}

pub async fn get_user_id_from_api(api_token: &str) -> anyhow::Result<Uuid> {
    let full_path = get_full_path("member");
    // cant do with api client as it isnt instantiated at the point of calling this
    let response = Client::new()
        .get(&full_path)
        .header("Shortcut-Token", api_token)
        .header("Content-Type", "application/json")
        .send()
        .await
        .with_context(|| format!("Failed to GET {}", &full_path))?;

    let user = response.json::<Member>().await?;

    Ok(user.id)
}
