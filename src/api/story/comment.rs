use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct StoryComment {
    author_id: Uuid,
    deleted: bool,
    // numerical position of comment oldest -> newest
    position: i32,
    text: Option<String>,
    // TODO: show replies in threads/nested
    // parent_id: Option<i32>
}
