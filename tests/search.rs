use klipy::Klipy;



fn client() -> Klipy {
    dotenvy::dotenv().ok();
    let key = std::env::var("KLIPY_API_KEY").expect("KLIPY_API_KEY not set");
    Klipy::new(key)
}

#[tokio::test]
async fn search_suggestions() {
    let results = client().search_suggestions("hello", 10).await.unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().all(|s| !s.is_empty()));
}

#[tokio::test]
async fn autocomplete() {
    let results = client().autocomplete("he", 10).await.unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().all(|s| !s.is_empty()));
}
