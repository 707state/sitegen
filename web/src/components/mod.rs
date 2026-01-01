pub mod archive_view;
pub mod card;
pub mod error_view;
pub mod home_view;
pub mod loading_view;
pub mod page;
pub mod post_view;
pub mod search_view;
pub mod topic_card;
use chrono::NaiveDate;
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TocItem {
    pub title: String,
    pub path: String,
    pub date: NaiveDate,
}
