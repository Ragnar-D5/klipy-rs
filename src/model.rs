use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Generic envelope wrapping every response.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Envelope<T> {
    pub result: bool,
    pub data: Option<T>,
}

/// A single concrete media file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
}

/// The set of formats available at a given size.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Formats {
    pub gif: Option<File>,
    pub webp: Option<File>,
    pub jpg: Option<File>,
    pub mp4: Option<File>,
    pub webm: Option<File>,
    pub png: Option<File>,
}

/// Renditions of a media item across its available sizes.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Sizes {
    pub hd: Option<Formats>,
    pub md: Option<Formats>,
    pub sm: Option<Formats>,
    pub xs: Option<Formats>,
}

/// A GIF, sticker, meme, or emoji item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MediaItem {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub file: Sizes,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub blur_preview: Option<String>,
}

/// Width/height pair for a clip rendition.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// Direct URLs for a clip's renditions.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ClipFiles {
    pub mp4: Option<String>,
    pub gif: Option<String>,
    pub webp: Option<String>,
}

/// A movie/video clip item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClipItem {
    pub url: String,
    pub title: String,
    pub slug: String,
    pub file: ClipFiles,
    #[serde(default)]
    pub file_meta: BTreeMap<String, Dimensions>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub blur_preview: Option<String>,
}

/// An advertisement object. If ads are enabled for your app, some API
/// responses may include an advertisement object alongside content objects.
/// When `type` is `"ad"`, the object is an advertisement.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdItem {
    /// HTML content to display (webview for mobile apps).
    pub content: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "type")]
    pub kind: String,
}

/// The generated emoji data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneratedEmoji {
    pub base64_encoded: String,
    pub mime_type: String,
}

/// Status of an AI Emoji generation request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationStatus {
    Processing,
    Success,
    Failed,
}

/// Status and result of an AI Emoji generation request.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmojiStatus {
    pub id: String,
    pub status: GenerationStatus,
    pub result: Option<GeneratedEmoji>,
}

/// A list entry: either content of type `T` or an advertisement.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Item<T> {
    Content(T),
    Ad(AdItem),
}

impl<T> Item<T> {
    /// Returns the content item, or `None` if this entry is an advertisement.
    pub fn content(&self) -> Option<&T> {
        match self {
            Item::Content(c) => Some(c),
            Item::Ad(_) => None,
        }
    }

    /// Returns the advertisement, or `None` if this entry is content.
    pub fn ad(&self) -> Option<&AdItem> {
        match self {
            Item::Content(_) => None,
            Item::Ad(a) => Some(a),
        }
    }
}

/// Metadata returned alongside AI Emoji list responses.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmojiMeta {
    /// Minimum display width for each emoji item, in pixels.
    pub item_min_width: Option<u32>,
    /// Maximum percentage by which an ad may be resized.
    pub ad_max_resize_percent: Option<u32>,
}

/// A paginated list of items. Pagination fields are absent for the
/// `items` endpoint. The `meta` field is present only on
/// AI Emoji list responses.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Page<T> {
    pub data: Vec<Item<T>>,
    pub current_page: Option<u32>,
    pub per_page: Option<u32>,
    #[serde(default)]
    pub has_next: bool,
    pub meta: Option<EmojiMeta>,
}

impl<T> Page<T> {
    /// Iterate over content items only, skipping any advertisements.
    pub fn content_items(&self) -> impl Iterator<Item = &T> {
        self.data.iter().filter_map(|i| i.content())
    }

    /// Iterate over advertisements only, skipping any content items.
    pub fn ad_items(&self) -> impl Iterator<Item = &AdItem> {
        self.data.iter().filter_map(|i| i.ad())
    }
}

/// A curated content category.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub category: String,
    pub query: String,
    pub preview_url: Option<String>,
}

/// The payload returned by a `categories` request.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Categories {
    pub locale: String,
    pub categories: Vec<Category>,
}
