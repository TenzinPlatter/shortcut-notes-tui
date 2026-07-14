use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Branch {
    id: i32,
    name: String,
}
