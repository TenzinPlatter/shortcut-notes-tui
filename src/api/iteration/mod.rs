use anyhow::Context;
use chrono::NaiveDate;
use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiClient, story::Story},
    custom_list::LinearListItem,
    dbg_file,
};

#[derive(Deserialize)]
struct IterationSearchResults {
    data: Vec<Iteration>,
    next: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IterationStatus {
    Unstarted,
    Started,
    Done,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Iteration {
    pub id: i32,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub app_url: String,
    pub status: IterationStatus,
}

impl LinearListItem for Iteration {
    fn id(&self) -> i32 { self.id }
    fn label(&self) -> &str { &self.name }
}

impl ApiClient {
    pub async fn get_current_iterations(&self) -> anyhow::Result<Vec<Iteration>> {
        let mut all = Vec::new();
        let mut next_path: Option<String> = None;

        loop {
            let response = match &next_path {
                None => {
                    let params = [
                        ("query", "is:started"),
                        ("page_size", "250"),
                        ("detail", "slim"),
                    ];
                    self.get_with_query("search/iterations", &params).await?
                }
                Some(path) => self.get_absolute_path(path).await?,
            };

            let page: IterationSearchResults = response
                .json()
                .await
                .context("Failed to parse search/iterations response")?;

            all.extend(page.data);
            match page.next {
                Some(n) => next_path = Some(n),
                None => break,
            }
        }

        Ok(all)
    }

    pub async fn get_all_iterations(&self) -> anyhow::Result<Vec<Iteration>> {
        let response = self.get("iterations").await?;
        Ok(response.json().await?)
    }

    pub async fn get_owned_iteration_stories(
        &self,
        iteration_ids: Vec<i32>,
    ) -> anyhow::Result<Vec<Story>> {
        let iteration_stories = join_all(iteration_ids.iter().map(|id| async move {
            let query = format!("iteration:{} owner:{}", id, self.mention_name);
            self.search_stories_all_pages(&query).await
        }))
        .await
        .into_iter()
        .filter_map(|res| match res {
            Ok(stories) => Some(stories),
            Err(e) => {
                dbg_file!("Failed to fetch iteration stories with error: {}", e);
                None
            }
        })
        .flatten()
        .collect();

        Ok(iteration_stories)
    }
}
