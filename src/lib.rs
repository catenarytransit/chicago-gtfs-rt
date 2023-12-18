use gtfs_rt::FeedMessage;
use std::{error::Error, io::Error};
use inline_colorization::*;
use serde::{Deserialize};

#[derive(Debug)]
pub struct ChicagoResults {
    vehicle_positions: FeedMessage,
    trip_updates: FeedMessage,
}

#[derive(Deserialize)]
struct TTPos {
    ctatt: TTPosInner
} 

#[derive(Deserialize)]
struct TTPosInner {
    tmst: String,
    errCd: String,
    errNm: Option<String>,
    route: Vec<TTPosRoute>
}

#[derive(Deserialize)]
struct TTPosRoute {
    //named @name
    #[serde(rename(deserialize = "@name"))]
    route_name: String,
    train: Vec<TTPosTrain>
}

#[derive(Deserialize)]
struct TTPosTrain {
    rn: String,
    #[serde(rename(deserialize = "destSt"))]
    dest_st: String,
    #[serde(rename(deserialize = "destNm"))]
    dest_nm: String,
    #[serde(rename(deserialize = "trDr"))]
    tr_dr: String,
    #[serde(rename(deserialize = "nextStaId"))]
    next_sta_id: String,
    #[serde(rename(deserialize = "nextStpId"))]
    next_stp_id: String,
    #[serde(rename(deserialize = "nextStaNm"))]
    next_sta_nm: String,
    prdt: String,
    #[serde(rename(deserialize = "arrT"))]
    arrt: String,
    #[serde(rename(deserialize = "isApp"))]
    is_app: String,
    #[serde(rename(deserialize = "isDly"))]
    is_dly: String,
    lat: String,
    lon: String,
    heading: String
}

const alltrainlines: &str = "Red,P,Y,Blue,Pink,G,Org,Brn";

pub async fn train_feed(
    client: reqwest::Client,
    key: &str,
) -> Result<ChicagoResults, Box<dyn std::error::Error>> {
    println!("running func");

    let response = client
         .get("https://www.transitchicago.com/api/1.0/ttpositions.aspx")
        .query(&[
            ("key", &key),
            ("rt", &alltrainlines),
            ("outputType", &"JSON"),
        ])
        .send()
        .await;

    if response.is_err() {
        let response = response.as_ref().unwrap_err();
        println!("{color_magenta}{:#?}{color_reset}", response.url().unwrap().as_str());
        println!("{:#?}", response);
    }

    match response {
        Ok(response) => {
            //println!("{color_magenta}{:#?}{color_reset}", response.url().as_str());
            //println!("{:?}", response.text().await);
            let text = response.text().await;
            match text {
                Ok(text) => { 
                    let json_output = serde_json::from_str::<TTPos>(text.as_str());

                    Err(Box::new(
                        std::io::Error::new(std::io::ErrorKind::Unsupported,"not implemented yet")
                    ))
                },
                Err(text) => {
                    Err(Box::new(
                        text,
                    ))
                }
            }
        }
        Err(err) => Err(Box::new("NaN".parse::<u32>().unwrap_err())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_train_feed() {
        let train_feeds = train_feed(
            reqwest::ClientBuilder::new()
                .use_rustls_tls()
                .deflate(true)
                .gzip(true)
                .brotli(true)
                .build()
                .unwrap(),
            "13f685e4b9054545b19470556103ec73",
        )
        .await;

        assert!(train_feeds.is_ok());
    }
}
