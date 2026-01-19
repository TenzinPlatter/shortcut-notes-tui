use serde::Deserialize;

#[derive(Deserialize)]
pub struct Branch {
    id: i32,
    name: String,
}
