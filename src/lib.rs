//! Async client for the [KLIPY](https://klipy.com) API
//!
//! ```no_run
//! # async fn run() -> klipy::Result<()> {
//! use klipy::{Klipy, ContentFilter};
//!
//! let klipy = Klipy::new("YOUR_APP_KEY");
//! let page = klipy
//!     .gifs()
//!     .trending()
//!     .per_page(20)
//!     .content_filter(ContentFilter::Medium)
//!     .send()
//!     .await?;
//! for item in page.content_items() {
//!     println!("{}", item.slug);
//! }
//! # Ok(()) }
//! ```

mod endpoints;
mod error;
mod model;
mod params;

pub use endpoints::{AdParams, Endpoints, ListBuilder, ReportBuilder, ShareBuilder};
pub use error::{Error, Result};
pub use model::{
    AdItem, Categories, Category, ClipFiles, ClipItem, Dimensions, GenerationStatus,
    EmojiMeta, EmojiStatus, File, Formats, GeneratedEmoji, Item, MediaItem, Page, Sizes,
};
pub use params::{ContentFilter, Format, ReportReason};

use model::Envelope;
use serde::de::DeserializeOwned;

const DEFAULT_BASE_URL: &str = "https://api.klipy.com";

/// The KLIPY API client.
#[derive(Debug, Clone)]
pub struct Klipy {
    http: reqwest::Client,
    base_url: String,
    app_key: String,
}

impl Klipy {
    /// Create a client with default options.
    pub fn new(app_key: impl Into<String>) -> Self {
        Self::builder(app_key).build()
    }

    /// Build a client with custom options.
    pub fn builder(app_key: impl Into<String>) -> KlipyBuilder {
        KlipyBuilder {
            app_key: app_key.into(),
            http: None,
            base_url: None,
            user_agent: None,
        }
    }

    /// GIF endpoints.
    pub fn gifs(&self) -> Endpoints<'_, MediaItem> {
        Endpoints::new(self, "gifs")
    }

    /// Sticker endpoints.
    pub fn stickers(&self) -> Endpoints<'_, MediaItem> {
        Endpoints::new(self, "stickers")
    }

    /// Clip endpoints. Note: the clips library is not yet fully MPA-rated.
    pub fn clips(&self) -> Endpoints<'_, ClipItem> {
        Endpoints::new(self, "clips")
    }

    /// Meme endpoints.
    pub fn memes(&self) -> Endpoints<'_, MediaItem> {
        Endpoints::new(self, "static-memes")
    }

    /// AI Emoji endpoints.
    pub fn emojis(&self) -> Endpoints<'_, MediaItem> {
        Endpoints::new(self, "emojis")
    }

    /// Generate an AI Emoji from a text `prompt` (max 300 chars). Returns a
    /// generation ID immediately; the emoji is processed asynchronously. Poll
    /// [`emoji_status`](Self::emoji_status) or provide a `callback_url` to
    /// receive the result via webhook.
    pub async fn generate_emoji(
        &self,
        prompt: impl Into<String>,
        callback_url: Option<String>,
    ) -> Result<String> {
        #[derive(serde::Serialize)]
        struct Body {
            prompt: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            callback_url: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct GenerateData {
            id: String,
        }
        let data: GenerateData = self
            .send(
                self.http
                    .post(self.url("emojis/generate"))
                    .json(&Body { prompt: prompt.into(), callback_url }),
            )
            .await?;
        Ok(data.id)
    }

    /// Check the status of an AI Emoji generation request. Status is one of
    /// `processing`, `success`, or `failed`.
    pub async fn emoji_status(&self, id: impl AsRef<str>) -> Result<EmojiStatus> {
        self.send(self.http.get(self.url(&format!("emojis/generated/{}", id.as_ref()))))
            .await
    }

    /// Related search terms for a query. `limit`: 1-50, default 10.
    pub async fn search_suggestions(&self, q: impl AsRef<str>, limit: u32) -> Result<Vec<String>> {
        self.send(
            self.http
                .get(self.url(&format!("search-suggestions/{}", q.as_ref())))
                .query(&[("limit", limit)]),
        )
        .await
    }

    /// Completed search terms for a partial query. `limit`: 1-50, default 10.
    pub async fn autocomplete(&self, q: impl AsRef<str>, limit: u32) -> Result<Vec<String>> {
        self.send(
            self.http
                .get(self.url(&format!("autocomplete/{}", q.as_ref())))
                .query(&[("limit", limit)]),
        )
        .await
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}/api/v1/{}/{}", self.base_url, self.app_key, path)
    }

    pub(crate) async fn send<T: DeserializeOwned>(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Result<T> {
        let resp = req.send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(Error::Status {
                status,
                body: resp.text().await.unwrap_or_default(),
            });
        }
        let env: Envelope<T> = resp.json().await?;
        match (env.result, env.data) {
            (true, Some(data)) => Ok(data),
            _ => Err(Error::Unsuccessful),
        }
    }

    pub(crate) async fn send_ack(&self, req: reqwest::RequestBuilder) -> Result<()> {
        let resp = req.send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(Error::Status {
                status,
                body: resp.text().await.unwrap_or_default(),
            });
        }
        #[derive(serde::Deserialize)]
        struct Ack {
            result: bool,
        }
        let ack: Ack = resp.json().await?;
        if ack.result { Ok(()) } else { Err(Error::Unsuccessful) }
    }
}

/// Builder for [`Klipy`].
#[derive(Debug)]
pub struct KlipyBuilder {
    app_key: String,
    http: Option<reqwest::Client>,
    base_url: Option<String>,
    user_agent: Option<String>,
}

impl KlipyBuilder {
    /// Set a default `User-Agent` for all requests. Required for ad delivery.
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Use a pre-configured [`reqwest::Client`]. Takes precedence over
    /// [`user_agent`](Self::user_agent).
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }

    /// Override the default base URL.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn build(self) -> Klipy {
        let http = self.http.unwrap_or_else(|| {
            let mut b = reqwest::Client::builder();
            if let Some(ua) = self.user_agent {
                b = b.user_agent(ua);
            }
            b.build().unwrap_or_default()
        });
        Klipy {
            http,
            base_url: self.base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            app_key: self.app_key,
        }
    }
}
