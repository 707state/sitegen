pub mod about_me_view;
pub mod archive_view;
pub mod card;
pub mod cover;
pub mod draggable_toc;
pub mod error_view;
pub mod home_view;
pub mod loading_view;
pub mod page;
pub mod post_view;
pub mod search_view;
pub mod topic_card;
pub mod webgl_cube;
pub mod webgl_ring;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FrontMatter {
    pub title: String,
    pub author: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub series: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PostPayload {
    pub path: String,
    pub modified_at_unix: Option<u64>,
    pub metadata: FrontMatter,
    #[serde(default)]
    pub headings: Vec<PostHeading>,
    #[serde(default)]
    pub math: bool,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PostHeading {
    pub level: u8,
    pub text: String,
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TocItem {
    pub title: String,
    pub path: String,
    pub date: NaiveDate,
    #[serde(default)]
    pub route: String,
}
