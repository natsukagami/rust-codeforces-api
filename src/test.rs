use super::*;

#[test]
pub fn user_works() {
    let http = reqwest::blocking::Client::new();

    // user.info
    User::info(&http, &["natsukagami", "vjudge2"]).unwrap();
    // user.rating
    User::rating(&http, "natsukagami").unwrap();
    // user.status
    User::status(&http, "natsukagami", 0, 1000).unwrap();
}


#[test]
pub fn user_listing_works() {
    let http = reqwest::blocking::Client::new();

    // user.rated_list
    User::rated_list(&http, false).unwrap();
}

#[test]
pub fn contest_listing_works() {
    let http = reqwest::blocking::Client::new();

    // contest.list
Contest::list(&http, true).unwrap();
}

#[test]
pub fn contest_works() {
    let http = reqwest::blocking::Client::new();

    // contest.standings
    Contest::standings(&http, 566, |f| f).unwrap();
}
