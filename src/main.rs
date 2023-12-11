use chicago_gtfs_rt::{self, train_feed};

#[tokio::main]
async fn main() {
    let train_feeds = train_feed(reqwest::Client::new(),"13f685e4b9054545b19470556103ec73").await;

    println!("{:?}", train_feeds);
}