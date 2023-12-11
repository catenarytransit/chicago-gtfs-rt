use chicago_gtfs_rt::{self, train_feed};

#[tokio::main]
async fn main() {
    let train_feeds = train_feed(reqwest::Client::new(),"Det2nqw85D8TqxqF6SpcYYjfu").await;

    println!("{:?}", train_feeds);
}