use klipy::Klipy;


const CUSTOMER: &str = "test-user-1";

fn client() -> Klipy {
    dotenvy::dotenv().ok();
    let key = std::env::var("KLIPY_API_KEY").expect("KLIPY_API_KEY not set");
    Klipy::new(key)
}

#[tokio::test]
async fn trending() {
    let page = client().stickers().trending().per_page(5).send().await.unwrap();
    assert!(!page.data.is_empty());
    let item = page.content_items().next().unwrap();
    assert!(!item.slug.is_empty());
    // Stickers have png format
    let hd = item.file.hd.as_ref().unwrap();
    assert!(hd.png.is_some());
}

#[tokio::test]
async fn search() {
    let page = client()
        .stickers()
        .search("happy")
        .per_page(5)
        .send()
        .await
        .unwrap();
    assert!(!page.data.is_empty());
}

#[tokio::test]
async fn categories() {
    let cats = client().stickers().categories("en_US").await.unwrap();
    assert!(!cats.categories.is_empty());
}

#[tokio::test]
async fn items() {
    let page = client().stickers().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    let items = client().stickers().items([&slug]).await.unwrap();
    assert_eq!(items.content_items().next().unwrap().slug, slug);
}

#[tokio::test]
async fn share() {
    let page = client().stickers().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    client()
        .stickers()
        .share(&slug)
        .customer_id(CUSTOMER)
        .send()
        .await
        .unwrap();
}


#[tokio::test]
async fn recent_and_hide() {
    let page = client().stickers().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    client()
        .stickers()
        .share(&slug)
        .customer_id(CUSTOMER)
        .send()
        .await
        .unwrap();
    let _ = client().stickers().recent(CUSTOMER).send().await.unwrap();
    client().stickers().hide_recent(CUSTOMER, &slug).await.unwrap();
}
