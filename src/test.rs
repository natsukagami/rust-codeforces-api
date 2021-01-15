use super::*;

#[tokio::test]
pub async fn user_works() {
    let http = Client::new();

    // user.info
    User::info(&http, &["natsukagami", "vjudge2"])
        .await
        .unwrap();
    // user.rating
    User::rating(&http, "natsukagami").await.unwrap();
    // user.status
    User::status(&http, "natsukagami", 0, 1000).await.unwrap();
}

#[tokio::test]
pub async fn user_listing_works() {
    let http = Client::new();

    // user.rated_list
    User::rated_list(&http, false).await.unwrap();
}

#[tokio::test]
pub async fn contest_listing_works() {
    let http = Client::new();

    // contest.list
    Contest::list(&http, true).await.unwrap();
}

#[tokio::test]
pub async fn contest_works() {
    let http = Client::new();

    // contest.standings
    Contest::standings(&http, 566, |f| f).await.unwrap();
}
