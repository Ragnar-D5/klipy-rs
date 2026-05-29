use std::marker::PhantomData;

use serde::de::DeserializeOwned;

use crate::model::{Categories, Page};
use crate::params::{ContentFilter, Format, ReportReason};
use crate::{Klipy, Result};

/// Unified endpoint builders for different content kinds.
#[derive(Debug, Clone, Copy)]
pub struct Endpoints<'a, T> {
    client: &'a Klipy,
    kind: &'static str,
    _item: PhantomData<fn() -> T>,
}

impl<'a, T: DeserializeOwned> Endpoints<'a, T> {
    pub(crate) fn new(client: &'a Klipy, kind: &'static str) -> Self {
        Self { client, kind, _item: PhantomData }
    }

    /// Trending items, updated throughout the day and localized by language
    /// and region.
    pub fn trending(&self) -> ListBuilder<'a, T> {
        ListBuilder::new(self.client, format!("{}/trending", self.kind))
    }

    /// Search by keyword. Results are ranked by relevance, popularity, and
    /// language context with fuzzy matching support.
    pub fn search(&self, q: impl Into<String>) -> ListBuilder<'a, T> {
        let mut b = ListBuilder::new(self.client, format!("{}/search", self.kind));
        b.query.push(("q".into(), q.into()));
        b
    }

    /// Items recently used by a specific user. Pass a stable `customer_id`
    /// to fetch per-user history.
    pub fn recent(&self, customer_id: impl AsRef<str>) -> ListBuilder<'a, T> {
        ListBuilder::new(
            self.client,
            format!("{}/recent/{}", self.kind, customer_id.as_ref()),
        )
    }

    /// Fetch specific items by slug.
    pub async fn items<I, S>(&self, slugs: I) -> Result<Page<T>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let joined = slugs
            .into_iter()
            .map(|s| s.as_ref().to_owned())
            .collect::<Vec<_>>()
            .join(",");
        let url = self.client.url(&format!("{}/items", self.kind));
        self.client
            .send(self.client.http().get(url).query(&[("slugs", joined)]))
            .await
    }

    /// Remove an item from a user's Recent list.
    pub async fn hide_recent(
        &self,
        customer_id: impl AsRef<str>,
        slug: impl AsRef<str>,
    ) -> Result<()> {
        let url = self
            .client
            .url(&format!("{}/recent/{}", self.kind, customer_id.as_ref()));
        self.client
            .send_ack(
                self.client
                    .http()
                    .delete(url)
                    .query(&[("slug", slug.as_ref())]),
            )
            .await
    }

    /// Log a share event for analytics and personalization.
    pub fn share(&self, slug: impl AsRef<str>) -> ShareBuilder<'a> {
        ShareBuilder {
            client: self.client,
            url: self.client.url(&format!("{}/share/{}", self.kind, slug.as_ref())),
            customer_id: None,
            q: None,
        }
    }

    /// Report an item flagged by a user.
    pub fn report(&self, slug: impl AsRef<str>, reason: ReportReason) -> ReportBuilder<'a> {
        ReportBuilder {
            client: self.client,
            url: self.client.url(&format!("{}/report/{}", self.kind, slug.as_ref())),
            customer_id: None,
            reason,
        }
    }

    /// Curated categories for this content kind. `locale` is `xx_YY` format
    /// (e.g. `"en_US"`).
    pub async fn categories(&self, locale: impl AsRef<str>) -> Result<Categories> {
        let url = self.client.url(&format!("{}/categories", self.kind));
        self.client
            .send(self.client.http().get(url).query(&[("locale", locale.as_ref())]))
            .await
    }
}

/// Builder for paginated list endpoints.
#[derive(Debug)]
pub struct ListBuilder<'a, T> {
    client: &'a Klipy,
    path: String,
    query: Vec<(String, String)>,
    _item: PhantomData<fn() -> T>,
}

impl<'a, T: DeserializeOwned> ListBuilder<'a, T> {
    fn new(client: &'a Klipy, path: String) -> Self {
        Self { client, path, query: Vec::new(), _item: PhantomData }
    }

    /// Page number (min 1, default 1).
    pub fn page(mut self, page: u32) -> Self {
        self.query.push(("page".into(), page.to_string()));
        self
    }

    /// Items per page.
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.query.push(("per_page".into(), per_page.to_string()));
        self
    }

    /// Stable user identifier for personalization.
    pub fn customer_id(mut self, customer_id: impl Into<String>) -> Self {
        self.query.push(("customer_id".into(), customer_id.into()));
        self
    }

    /// ISO 3166 Alpha-2 country/language code (e.g. `"us"`, `"ru"`).
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.query.push(("locale".into(), locale.into()));
        self
    }

    /// Content safety filter level.
    pub fn content_filter(mut self, filter: ContentFilter) -> Self {
        self.query.push(("content_filter".into(), filter.as_str().into()));
        self
    }

    /// Restrict results to specific formats only.
    pub fn format_filter(mut self, formats: impl IntoIterator<Item = Format>) -> Self {
        let joined = formats.into_iter().map(Format::as_str).collect::<Vec<_>>().join(",");
        self.query.push(("format_filter".into(), joined));
        self
    }

    /// Attach ad request parameters. Requires a browser-like
    /// `User-Agent` set via [`KlipyBuilder::user_agent`](crate::KlipyBuilder::user_agent).
    pub fn ad_params(mut self, params: AdParams) -> Self {
        self.query.extend(params.into_query());
        self
    }

    /// Execute the request.
    pub async fn send(self) -> Result<Page<T>> {
        let url = self.client.url(&self.path);
        self.client
            .send(self.client.http().get(url).query(&self.query))
            .await
    }
}

/// Builder for the share-trigger endpoint.
#[derive(Debug)]
pub struct ShareBuilder<'a> {
    client: &'a Klipy,
    url: String,
    customer_id: Option<String>,
    q: Option<String>,
}

impl<'a> ShareBuilder<'a> {
    /// Stable user identifier.
    pub fn customer_id(mut self, customer_id: impl Into<String>) -> Self {
        self.customer_id = Some(customer_id.into());
        self
    }

    /// The search query that led to this share. Required for Search API shares.
    pub fn query(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }

    /// Execute the request.
    pub async fn send(self) -> Result<()> {
        let body = serde_json::json!({
            "customer_id": self.customer_id.unwrap_or_default(),
            "q": self.q.unwrap_or_default(),
        });
        self.client.send_ack(self.client.http().post(self.url).json(&body)).await
    }
}

/// Builder for the report endpoint.
#[derive(Debug)]
pub struct ReportBuilder<'a> {
    client: &'a Klipy,
    url: String,
    customer_id: Option<String>,
    reason: ReportReason,
}

impl<'a> ReportBuilder<'a> {
    /// Stable user identifier.
    pub fn customer_id(mut self, customer_id: impl Into<String>) -> Self {
        self.customer_id = Some(customer_id.into());
        self
    }

    /// Execute the request.
    pub async fn send(self) -> Result<()> {
        let body = serde_json::json!({
            "customer_id": self.customer_id.unwrap_or_default(),
            "reason": self.reason,
        });
        self.client.send_ack(self.client.http().post(self.url).json(&body)).await
    }
}

/// Ad request parameters for list endpoints.
/// Required options: `customer_id`, `min_width`, `max_width`, `min_height`, `max_height`.
#[derive(Debug, Default)]
pub struct AdParams {
    /// Unique user ID (required).
    pub customer_id: Option<String>,
    /// Minimum ad width in pixels (recommended: 50).
    pub min_width: Option<u32>,
    /// Maximum ad width in pixels (recommended: device width).
    pub max_width: Option<u32>,
    /// Minimum ad height in pixels (recommended: 50).
    pub min_height: Option<u32>,
    /// Maximum ad height in pixels (recommended: 250).
    pub max_height: Option<u32>,
    /// Ad position in the response array (0-based). Defaults to client setting
    /// or a random value between 0 and 2.
    pub position: Option<u32>,
    /// `true` to receive an iframe URL instead of HTML content.
    pub iframe: Option<bool>,
    /// App version (e.g. `"10.9.3"`).
    pub app_version: Option<String>,
    /// OS name
    pub os: Option<String>,
    /// OS version (e.g. `"18.0"`).
    pub os_version: Option<String>,
    /// Hardware version (e.g. `"15"`).
    pub hardware_version: Option<String>,
    /// Device make
    pub make: Option<String>,
    /// Device model
    pub model: Option<String>,
    /// Advertiser ID: IDFA (iOS), GPID (Android), unhashed.
    pub ifa: Option<String>,
    /// Physical screen height in pixels.
    pub device_h: Option<u32>,
    /// Physical screen width in pixels.
    pub device_w: Option<u32>,
    /// Pixels per inch.
    pub ppi: Option<u32>,
    /// Physical-to-device-independent pixel ratio.
    pub px_ratio: Option<f32>,
    /// Language, ISO-639-1-alpha-2 (e.g. `"EN"`).
    pub language: Option<String>,
    /// Carrier or ISP.
    pub carrier: Option<String>,
    /// MCC-MNC code (e.g. `"310-005"`).
    pub mcc_mnc: Option<String>,
    /// Connection type: 0=unknown, 1=ethernet, 2=WiFi, 3=cellular(unknown),
    /// 4=2G, 5=3G, 6=4G, 7=5G.
    pub connection_type: Option<u32>,
    /// IMEI hashed via SHA1.
    pub did_sha1: Option<String>,
    /// IMEI hashed via MD5.
    pub did_md5: Option<String>,
    /// Platform device ID hashed via SHA1.
    pub dpid_sha1: Option<String>,
    /// Platform device ID hashed via MD5.
    pub dpid_md5: Option<String>,
    /// MAC address hashed via SHA1.
    pub mac_sha1: Option<String>,
    /// MAC address hashed via MD5.
    pub mac_md5: Option<String>,
    /// User year of birth (4-digit).
    pub year_of_birth: Option<u32>,
    /// User gender: `"M"`, `"F"`, or `"O"`.
    pub gender: Option<String>,
}

impl AdParams {
    fn into_query(self) -> Vec<(String, String)> {
        let mut q: Vec<(String, String)> = Vec::new();
        macro_rules! push {
            ($key:expr, $val:expr) => {
                if let Some(v) = $val {
                    q.push(($key.into(), v.to_string()));
                }
            };
        }
        push!("customer_id", self.customer_id);
        push!("ad-min-width", self.min_width);
        push!("ad-max-width", self.max_width);
        push!("ad-min-height", self.min_height);
        push!("ad-max-height", self.max_height);
        push!("ad-position", self.position);
        if let Some(v) = self.iframe {
            q.push(("ad-iframe".into(), if v { "1" } else { "0" }.into()));
        }
        push!("ad-app-version", self.app_version);
        push!("ad-os", self.os);
        push!("ad-osv", self.os_version);
        push!("ad-hwv", self.hardware_version);
        push!("ad-make", self.make);
        push!("ad-model", self.model);
        push!("ad-ifa", self.ifa);
        push!("ad-device-h", self.device_h);
        push!("ad-device-w", self.device_w);
        push!("ad-ppi", self.ppi);
        push!("ad-pxratio", self.px_ratio);
        push!("ad-language", self.language);
        push!("ad-carrier", self.carrier);
        push!("ad-mccmnc", self.mcc_mnc);
        push!("ad-connection-type", self.connection_type);
        push!("ad-didsha1", self.did_sha1);
        push!("ad-didmd5", self.did_md5);
        push!("ad-dpidsha1", self.dpid_sha1);
        push!("ad-dpidmd5", self.dpid_md5);
        push!("ad-macsha1", self.mac_sha1);
        push!("ad-macmd5", self.mac_md5);
        push!("ad-yob", self.year_of_birth);
        push!("ad-gender", self.gender);
        q
    }
}
