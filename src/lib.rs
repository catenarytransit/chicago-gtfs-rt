use gtfs_rt::FeedMessage;
use std::error::Error;

#[derive(Debug)]
pub struct ChicagoResults {
    vehicle_positions: FeedMessage,
    trip_updates: FeedMessage
}

pub async fn train_feed(client: reqwest::Client, key: &str) -> Result<ChicagoResults,Box <dyn std::error::Error>> {
    
    println!("running func");

    let response = client.get("https://lapi.transitchicago.com/api/1.0/ttpositions.aspx?outputType=JSON")
    .query(&[("key", &key), ("rt", &"Pink")])
    .send().await;

    
    println!("{:?}", response);

    match response {
        Ok(response) => {
            println!("{:?}", response.text().await);
            Err(Box::new("9999999999999999999999999999999".parse::<u32>().unwrap_err()))
        },
        Err(err) => Err(Box::new("NaN".parse::<u32>().unwrap_err()))
    }

    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_train_feed() {
        let train_feeds = train_feed(reqwest::Client::new(),"Det2nqw85D8TqxqF6SpcYYjfu").await;

        assert!(train_feeds.is_ok());
    }
}
