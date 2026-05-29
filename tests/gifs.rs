use klipy::{ContentFilter, Klipy};


const CUSTOMER: &str = "test-user-1";

fn client() -> Klipy {
    dotenvy::dotenv().ok();
    let key = std::env::var("KLIPY_API_KEY").expect("KLIPY_API_KEY not set");
    Klipy::new(key)
}

#[tokio::test]
async fn trending() {
    let page = client().gifs().trending().per_page(5).send().await.unwrap();
    assert!(!page.data.is_empty());
    let item = page.content_items().next().unwrap();
    assert!(!item.slug.is_empty());
}

#[tokio::test]
async fn search() {
    let page = client()
        .gifs()
        .search("hello")
        .per_page(5)
        .content_filter(ContentFilter::Medium)
        .send()
        .await
        .unwrap();
    assert!(!page.data.is_empty());
}

#[tokio::test]
async fn categories() {
    let cats = client().gifs().categories("en_US").await.unwrap();
    assert!(!cats.categories.is_empty());
}

#[tokio::test]
async fn items() {
    let page = client().gifs().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    let items = client().gifs().items([&slug]).await.unwrap();
    assert_eq!(items.content_items().next().unwrap().slug, slug);
}

#[tokio::test]
async fn share() {
    let page = client().gifs().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    client()
        .gifs()
        .share(&slug)
        .customer_id(CUSTOMER)
        .send()
        .await
        .unwrap();
}


#[tokio::test]
async fn recent_and_hide() {
    let page = client().gifs().trending().per_page(1).send().await.unwrap();
    let slug = page.content_items().next().unwrap().slug.clone();
    client()
        .gifs()
        .share(&slug)
        .customer_id(CUSTOMER)
        .send()
        .await
        .unwrap();

    let recent = client()
        .gifs()
        .recent(CUSTOMER)
        .per_page(5)
        .send()
        .await
        .unwrap();
    let _ = recent.content_items().count();

    client()
        .gifs()
        .hide_recent(CUSTOMER, &slug)
        .await
        .unwrap();
}
