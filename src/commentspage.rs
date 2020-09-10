use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    pub data: CommentData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentData {
    pub children: Vec<CommentChildren>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentChildren {
    pub data: CommentChildrenData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentChildrenData {
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}
