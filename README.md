# klipy

<a href="https://crates.io/crates/klipy"><img src="https://img.shields.io/crates/v/klipy?style=for-the-badge"></a>
<a href="https://docs.rs/klipy"><img src="https://img.shields.io/badge/docs.rs-rustdoc-green?style=for-the-badge"></a>

Async Rust client for the [KLIPY API](https://docs.klipy.com)

This is an unofficial library and is not affiliated with Klipy in any way.

## Installation

```sh
cargo add klipy
```

## Usage

```rust
use klipy::{Klipy, ContentFilter};

let klipy = Klipy::new("YOUR_APP_KEY");

// trending GIFs
let page = klipy.gifs().trending().per_page(20).send().await?;
for item in page.content_items() {
    println!("{} - {}", item.slug, item.kind);
}

// search
let page = klipy.stickers().search("happy").content_filter(ContentFilter::Medium).send().await?;

// fetch by slug
let page = klipy.clips().items(["they-love-me-spider-man"]).await?;

// log a share
klipy.gifs().share("hello-hi-662").customer_id("user-uuid").send().await?;

// search suggestions
let suggestions = klipy.search_suggestions("hel", 10).await?;
```

## Ads

```rust
use klipy::{AdParams, KlipyBuilder};

let klipy = Klipy::builder("YOUR_APP_KEY")
    .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148")
    .build();

let page = klipy.gifs().trending()
    .ad_params(AdParams {
        customer_id: Some("user-uuid".into()),
        min_width: Some(50),
        max_width: Some(400),
        min_height: Some(50),
        max_height: Some(250),
        os: Some("ios".into()),
        ..Default::default()
    })
    .send()
    .await?;

for item in &page.data {
    match item {
        Item::Content(gif) => { /* display gif */ }
        Item::Ad(ad) => { /* render ad.content (either HTML or iFrame) */ }
    }
}
```

## Custom client

```rust
let klipy = Klipy::builder("YOUR_APP_KEY")
    .user_agent("...")
    .base_url("https://api.klipy.com") // optional override
    .build();
```

Or bring your `reqwest::Client`:

```rust
let klipy = Klipy::builder("YOUR_APP_KEY")
    .http_client(my_client)
    .build();
```

## Licensing

Dual licensed under MIT OR Apache-2.0 at your discretion. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
