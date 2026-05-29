use serde::Serialize;

/// Specify the content safety filter level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentFilter {
    Off,
    Low,
    Medium,
    High,
}

impl ContentFilter {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ContentFilter::Off => "off",
            ContentFilter::Low => "low",
            ContentFilter::Medium => "medium",
            ContentFilter::High => "high",
        }
    }
}

/// A desired format. Possible values: gif, webp, jpg, mp4, webm (stickers also
/// use png).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Gif,
    Webp,
    Jpg,
    Mp4,
    Webm,
    Png,
}

impl Format {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Format::Gif => "gif",
            Format::Webp => "webp",
            Format::Jpg => "jpg",
            Format::Mp4 => "mp4",
            Format::Webm => "webm",
            Format::Png => "png",
        }
    }
}

/// The reason for reporting the content, providing context for KLIPY's review
/// process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportReason {
    /// Nudity or sexually explicit content.
    Nudity,
    /// Graphic violence or violent behavior.
    Violence,
    /// Racist, homophobic, or hateful content.
    HateSpeech,
    /// Bullying, personal attacks, or targeted harassment.
    Harassment,
    /// Repetitive, irrelevant, or misleading content.
    Spam,
    /// False claims, misleading text, or manipulated media.
    Misinformation,
    /// Content believed to infringe on intellectual property rights.
    Copyright,
    /// Generally offensive or culturally inappropriate material.
    Offensive,
    /// Content that promotes or depicts illegal activity.
    Illegal,
    /// Content doesn't load, is corrupted, or is unplayable.
    Broken,
    /// Extremely low resolution or unreadable content.
    LowQuality,
    /// Content doesn't match the tag/query or is miscategorized.
    NotRelevant,
    /// Fake identity, misleading branding, or impersonation.
    Impersonation,
    /// Other issues not listed above. Free-text description recommended.
    Other,
}
