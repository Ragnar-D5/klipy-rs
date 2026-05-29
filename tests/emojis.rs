use klipy::Klipy;


const CUSTOMER: &str = "test-user-1";

fn client() -> Klipy {
    dotenvy::dotenv().ok();
    let key = std::env::var("KLIPY_API_KEY").expect("KLIPY_API_KEY not set");
    Klipy::new(key)
}

#[tokio::test]
async fn trending() {
    let page = client().emojis().trending().per_page(5).send().await.unwrap();
    assert!(!page.data.is_empty());
    let item = page.content_items().next().unwrap();
    assert!(!item.slug.is_empty());
    // Emojis use png/webp only
    let hd = item.file.hd.as_ref().unwrap();
    assert!(hd.png.is_some());
}

#[tokio::test]
async fn trending_has_meta() {
    let page = client().emojis().trending().per_page(1).send().await.unwrap();
    let meta = page.meta.as_ref().unwrap();
    assert!(meta.item_min_width.is_some());
}

#[tokio::test]
async fn search() {
    let page = client()
        .emojis()
        .search("love")
        .per_page(5)
        .send()
        .await
        .unwrap();
    assert!(!page.data.is_empty());
}

#[tokio::test]
async fn categories() {
    let cats = client().emojis().categories("en_US").await.unwrap();
    assert!(!cats.categories.is_empty());
}

#[tokio::test]
async fn items() {
    let page = client().emojis().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    let items = client().emojis().items([&slug]).await.unwrap();
    assert_eq!(items.content_items().next().unwrap().slug, slug);
}

#[tokio::test]
async fn share() {
    let page = client().emojis().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    client()
        .emojis()
        .share(&slug)
        .customer_id(CUSTOMER)
        .send()
        .await
        .unwrap();
}


#[tokio::test]
async fn recent_and_hide() {
    let page = client().emojis().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    client()
        .emojis()
        .share(&slug)
        .customer_id(CUSTOMER)
        .send()
        .await
        .unwrap();
    let _ = client().emojis().recent(CUSTOMER).send().await.unwrap();
    client().emojis().hide_recent(CUSTOMER, &slug).await.unwrap();
}

#[tokio::test]
async fn generate_and_status() {
    let id = client()
        .generate_emoji("a happy cat", None)
        .await
        .unwrap();
    assert!(!id.is_empty());

    let status = client().emoji_status(&id).await.unwrap();
    assert_eq!(status.id, id);
    assert!(matches!(
        status.status,
        klipy::GenerationStatus::Processing
            | klipy::GenerationStatus::Success
            | klipy::GenerationStatus::Failed
    ));
}
