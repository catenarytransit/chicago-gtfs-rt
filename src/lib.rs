use gtfs_realtime::trip_update::{StopTimeEvent, StopTimeUpdate};
use gtfs_realtime::{FeedEntity, FeedMessage};
use inline_colorization::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
#[derive(Debug, Clone)]
pub struct ChicagoResults {
    pub vehicle_positions: FeedMessage,
    pub trip_updates: FeedMessage,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct TTPos {
    ctatt: TTPosInner,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct TTPosInner {
    tmst: String,
    errCd: String,
    errNm: Option<String>,
    route: Vec<TTPosRoute>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct TTPosRoute {
    //named @name
    #[serde(rename(deserialize = "@name"))]
    route_name: String,
    train: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
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
    heading: String,
}

const alltrainlines: &str = "Red,P,Y,Blue,Pink,G,Org,Brn";

pub async fn train_feed(
    client: &reqwest::Client,
    key: &str,
    trips: &str,
    stoptimes: &str,
) -> Result<ChicagoResults, Box<dyn std::error::Error + Sync + Send>> {
    let mut trip_data_raw = csv::Reader::from_reader(trips.as_bytes());
    let trip_data = trip_data_raw.records();

    let mut stop_times_raw = csv::Reader::from_reader(stoptimes.as_bytes());
    let stop_times = stop_times_raw.records();

    let mut run_ids: HashMap<u16, String> = HashMap::new();

    for record in trip_data {
        let record = record?;
        if record[8].contains('R') {
            let final_run = record[8].replace("R", "");
            let final_trip = record[2].to_string();

            run_ids.insert(final_run.parse::<u16>().unwrap(), final_trip);
        }
    }

    let mut trip_stops: HashMap<String, Vec<StopTimeUpdate>> = HashMap::new();

    for record in stop_times {
        let record = record?;
        if record[3].starts_with("3") && record[3].chars().count() == 5 {
            let record_trip = &record[0];

            let record_update = StopTimeUpdate {
                stop_sequence: Some(record[4].parse::<u32>().unwrap()),
                stop_id: Some(record[3].to_string()),
                arrival: Some(StopTimeEvent {
                    delay: None,
                    uncertainty: None,
                    // time: Some(record[1].parse::<i64>().unwrap())
                    time: None,
                }),
                departure: Some(StopTimeEvent {
                    delay: None,
                    uncertainty: None,
                    // time: Some(record[2].parse::<i64>().unwrap())
                    time: None,
                }),
                departure_occupancy_status: None,
                schedule_relationship: None,
                stop_time_properties: None,
            };

            trip_stops
                .entry(record_trip.to_string())
                .or_insert(vec![record_update.clone()])
                .push(record_update);
        }
    }

    // trip_stops.iter().for_each(|(k, v)| println!("Trip {} calls at {}, {}, {}, and more stops", k, v.get(0).unwrap().stop_id.as_ref().unwrap().to_string(), v.get(1).unwrap().stop_id.as_ref().unwrap().to_string(), v.get(2).unwrap().stop_id.as_ref().unwrap().to_string() ));

    let response = client
        .get("https://www.transitchicago.com/api/1.0/ttpositions.aspx")
        .query(&[
            ("key", &key),
            ("rt", &alltrainlines),
            ("outputType", &"JSON"),
        ])
        .send()
        .await;

    if let Err(response) = &response {
        println!(
            "{color_magenta}{:#?}{color_reset}",
            response.url().unwrap().as_str()
        );
        //  println!("{:?}", response);
    }

    let response = response?;
    let text = response.text().await?;
    let json_output = serde_json::from_str::<TTPos>(text.as_str())?;

    //Vec<TTPosTrain> or TTPosTrain

    let mut train_positions: Vec<FeedEntity> = vec![];
    let mut trip_updates: Vec<FeedEntity> = vec![];

    for train_line_group in json_output.ctatt.route {
        if let Some(train_value) = train_line_group.train {
            let train_data_vec: Vec<TTPosTrain> = match &train_value {
                serde_json::Value::Object(train_map) => {
                    match serde_json::from_value::<TTPosTrain>(train_value) {
                        Err(_) => vec![],
                        Ok(valid_train_map) => vec![valid_train_map],
                    }
                }
                serde_json::Value::Array(train_map) => {
                    match serde_json::from_value::<Vec<TTPosTrain>>(train_value) {
                        Err(_) => vec![],
                        Ok(valid_train_map) => valid_train_map,
                    }
                }
                _ => vec![],
            };

            for train in &train_data_vec {
                let lat = train.lat.parse::<f32>();
                let lon = train.lon.parse::<f32>();

                let train_run_id = train.rn.parse::<u16>().unwrap();
                let train_trip_id = run_ids.get(&train_run_id);

                if train_trip_id.is_some() {
                    if let Ok(lat) = lat {
                        if let Ok(lon) = lon {
                            let pos_entity: FeedEntity = FeedEntity {
                                id: train.rn.clone(),
                                stop: None,
                                trip_modifications: None,
                                is_deleted: None,
                                trip_update: None,
                                vehicle: Some(gtfs_realtime::VehiclePosition {
                                    trip: Some(gtfs_realtime::TripDescriptor {
                                        modified_trip: None,
                                        trip_id: train_trip_id.cloned(),
                                        route_id: Some(capitalize(&train_line_group.route_name)),
                                        direction_id: Some(train.tr_dr.parse::<u32>().unwrap()),
                                        start_time: None,
                                        start_date: None,
                                        schedule_relationship: None,
                                    }),
                                    vehicle: Some(gtfs_realtime::VehicleDescriptor {
                                        id: Some(train.rn.clone()),
                                        label: None,
                                        license_plate: None,
                                        wheelchair_accessible: None,
                                    }),
                                    position: Some(gtfs_realtime::Position {
                                        latitude: lat,
                                        longitude: lon,
                                        bearing: match train.heading.parse::<f32>() {
                                            Ok(bearing) => Some(bearing),
                                            _ => None,
                                        },
                                        odometer: None,
                                        speed: None,
                                    }),
                                    current_status: None,
                                    current_stop_sequence: None,
                                    stop_id: None,
                                    timestamp: Some(
                                        SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .expect("Time went backwards")
                                            .as_secs(),
                                    ),
                                    congestion_level: None,
                                    occupancy_percentage: None,
                                    occupancy_status: None,
                                    multi_carriage_details: vec![],
                                }),
                                alert: None,
                                shape: None,
                            };

                            train_positions.push(pos_entity);

                            let trip_entity: FeedEntity = FeedEntity {
                                id: train.rn.clone(),
                                vehicle: None,
                                alert: None,
                                shape: None,
                                stop: None,
                                trip_modifications: None,
                                is_deleted: None,
                                trip_update: Some(gtfs_realtime::TripUpdate {
                                    trip: (gtfs_realtime::TripDescriptor {
                                        modified_trip: None,
                                        trip_id: train_trip_id.cloned(),
                                        route_id: Some(capitalize(&train_line_group.route_name)),
                                        direction_id: Some(train.tr_dr.parse::<u32>().unwrap()),
                                        start_time: None,
                                        start_date: None,
                                        schedule_relationship: None,
                                    }),
                                    vehicle: Some(gtfs_realtime::VehicleDescriptor {
                                        id: Some(train.rn.clone()),
                                        label: None,
                                        license_plate: None,
                                        wheelchair_accessible: None,
                                    }),
                                    stop_time_update: trip_stops
                                        .get(train_trip_id.expect("Trip ID matching failed"))
                                        .expect("Trip not found")
                                        .clone(),
                                    timestamp: Some(
                                        SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .expect("Time went backwards")
                                            .as_secs(),
                                    ),
                                    delay: None,
                                    trip_properties: None,
                                }),
                            };

                            trip_updates.push(trip_entity);
                        }
                    }
                }
            }
        }
    }

    Ok(ChicagoResults {
        vehicle_positions: gtfs_realtime::FeedMessage {
            entity: train_positions,
            header: gtfs_realtime::FeedHeader {
                timestamp: Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs(),
                ),
                gtfs_realtime_version: String::from("2.0"),
                incrementality: None,
            },
        },
        trip_updates: gtfs_realtime::FeedMessage {
            entity: trip_updates,
            header: gtfs_realtime::FeedHeader {
                timestamp: Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs(),
                ),
                gtfs_realtime_version: String::from("2.0"),
                incrementality: None,
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use std::{fs, io, path::Path};
    use zip::ZipArchive;

    #[tokio::test]
    async fn test_train_feed() {
        let trips_file_data = fs::read_to_string("static/trips.txt");
        let stoptimes_file_data = fs::read_to_string("static/stop_times.txt");

        let train_feeds = train_feed(
            &reqwest::ClientBuilder::new()
                .use_rustls_tls()
                .deflate(true)
                .gzip(true)
                .brotli(true)
                .build()
                .unwrap(),
            "13f685e4b9054545b19470556103ec73",
            &trips_file_data.expect("Bad trips file"),
            &stoptimes_file_data.expect("Bad stoptimes file"),
        )
        .await;

        assert!(train_feeds.is_ok());

        println!("{:#?}", train_feeds);
    }

    /*
    #[tokio::test]
    async fn test_bus_feed() {
        let api_key = "Det2nqw85D8TqxqF6SpcYYjfu";

        let bus = reqwest::get(
            "https://www.ctabustracker.com/bustime/api/v2/getvehicles?key=Det2nqw85D8TqxqF6SpcYYjfu&rt=1"
        ).await.unwrap().text().await.unwrap();

        println!("{}", bus);
    }*/
}
