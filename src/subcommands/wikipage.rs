use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WikiPageData {
    pub content_md: String,
    pub content_html: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WikiPage {
    pub data: WikiPageData,
}
