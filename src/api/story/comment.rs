use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone)]
pub struct StoryComment {
    author_id: Uuid,
    deleted: bool,
    // numerical position of comment oldest -> newest
    position: i32,
    text: Option<String>,
    // TODO: show replies in threads/nested
    // parent_id: Option<i32>
}
