pub mod card;
pub mod error_view;
pub mod home_view;
pub mod loading_view;
pub mod page;
pub mod post_view;
pub mod topic_card;
use serde::Deserialize;
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct FrontMatter {
    pub title: String,
    pub author: String,
    pub tags: Vec<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PostPayload {
    pub path: String,
    pub modified_at_unix: Option<u64>,
    pub metadata: FrontMatter,
    pub content: String,
}
