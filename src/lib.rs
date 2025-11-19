use chrono::Datelike;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use chrono_tz::America::Chicago;
use gtfs_realtime::alert::{Cause, Effect, SeverityLevel};
use gtfs_realtime::translated_string::Translation;
use core::time;
use gtfs_realtime::trip_update::stop_time_update::StopTimeProperties;
use gtfs_realtime::trip_update::{StopTimeEvent, StopTimeUpdate};
use gtfs_realtime::{Alert, EntitySelector, FeedEntity, FeedMessage, TimeRange, TranslatedString, stop};
use inline_colorization::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
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
    pub alerts: FeedMessage,
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

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct TTFollow {
    ctatt: TTFollowInner,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct TTFollowInner {
    tmst: String,
    #[serde(rename(deserialize = "errCd"))]
    err_cd: String,
    #[serde(rename(deserialize = "errNm"))]
    err_nm: Option<String>,
    position: Option<TTFollowPosition>,
    eta: Option<Vec<TTFollowPrediction>>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct TTFollowPosition {
    lat: String,
    lon: String,
    heading: String,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct TTFollowPrediction {
    #[serde(rename(deserialize = "staId"))]
    sta_id: String,
    #[serde(rename(deserialize = "stpId"))]
    stp_id: String,
    #[serde(rename(deserialize = "staNm"))]
    sta_nm: String,
    #[serde(rename(deserialize = "stpDe"))]
    stp_de: String,
    rn: String,
    rt: String,
    #[serde(rename(deserialize = "destSt"))]
    dest_st: String,
    #[serde(rename(deserialize = "destNm"))]
    dest_nm: String,
    #[serde(rename(deserialize = "trDr"))]
    tr_dr: String,
    prdt: String,
    #[serde(rename(deserialize = "arrT"))]
    arrt: String,
    #[serde(rename(deserialize = "isApp"))]
    is_app: String,
    #[serde(rename(deserialize = "isDly"))]
    is_dly: String,
    flags: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct CDATAWrapper {
    #[serde(rename(deserialize = "#cdata-section"))]
    cdata_section: String,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct CTAAlerts {
    #[serde(rename(deserialize = "CTAAlerts"))]
    cta_alerts: CTAAlertsInner,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct CTAAlertsInner {
    #[serde(rename(deserialize = "TimeStamp"))]
    time_stamp: String,
    #[serde(rename(deserialize = "ErrorCode"))]
    error_code: i64,
    #[serde(rename(deserialize = "ErrorMessage"))]
    error_message: Option<String>,
    #[serde(rename(deserialize = "Alert"))]
    alert: Vec<CTAAlert>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct CTAAlert {
    #[serde(rename(deserialize = "AlertId"))]
    alert_id: String,
    #[serde(rename(deserialize = "Headline"))]
    headline: String,
    #[serde(rename(deserialize = "ShortDescription"))]
    short_description: Option<String>,
    #[serde(rename(deserialize = "FullDescription"))]
    full_description: CDATAWrapper,
    #[serde(rename(deserialize = "SeverityScore"))]
    severity_score: String,
    #[serde(rename(deserialize = "SeverityColor"))]
    severity_color: String,
    #[serde(rename(deserialize = "SeverityCSS"))]
    severity_css: String,
    #[serde(rename(deserialize = "Impact"))]
    impact: String,
    #[serde(rename(deserialize = "EventStart"))]
    event_start: Option<String>,
    #[serde(rename(deserialize = "EventEnd"))]
    event_end: Option<String>,
    #[serde(rename(deserialize = "TBD"))]
    tbd: String,
    #[serde(rename(deserialize = "MajorAlert"))]
    major_alert: String,
    #[serde(rename(deserialize = "AlertURL"))]
    alert_url: CDATAWrapper,
    #[serde(rename(deserialize = "ImpactedService"))]
    impacted_service: CTAAlertImpactedService,
    #[serde(rename(deserialize = "ttim"))]
    ttim: String,
    #[serde(rename(deserialize = "GUID"))]
    guid: String,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct CTAAlertImpactedService {
    #[serde(rename(deserialize = "Service"))]
    service: Vec<CTAAlertImpactedServiceInner>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct CTAAlertImpactedServiceInner {
    #[serde(rename(deserialize = "ServiceType"))]
    service_type: String,
    #[serde(rename(deserialize = "ServiceTypeDescription"))]
    service_type_description: String,
    #[serde(rename(deserialize = "ServiceName"))]
    service_name: String,
    #[serde(rename(deserialize = "ServiceId"))]
    service_id: String,
    #[serde(rename(deserialize = "ServiceBackColor"))]
    service_back_color: String,
    #[serde(rename(deserialize = "ServiceTextColor"))]
    service_text_color: String,
    #[serde(rename(deserialize = "ServiceURL"))]
    service_url: CDATAWrapper,
}

struct InternalTripIdSearch {
    trip_id: String,
    service_date: chrono::NaiveDate,
    trip_start_time: chrono::DateTime<chrono_tz::Tz>,
}

fn current_chicago_time() -> chrono::DateTime<chrono_tz::Tz> {
    let utc = chrono::Utc::now().naive_utc();
    chrono_tz::America::Chicago.from_utc_datetime(&utc)
}

fn midnight_chicago_from_naive_date(
    input_date: chrono::NaiveDate,
) -> chrono::DateTime<chrono_tz::Tz> {
    let noon_chicago = chrono_tz::America::Chicago
        .ymd(input_date.year(), input_date.month(), input_date.day())
        .and_hms(12, 0, 0);

    let midnight = noon_chicago - chrono::Duration::hours(12);

    midnight
}

const alltrainlines: &str = "Red,P,Y,Blue,Pink,G,Org,Brn";

fn timestamp_from_str(timestamp: &str) -> Option<i64> {
    let naive_time = chrono::NaiveDateTime::parse_from_str(&timestamp, "%Y-%m-%dT%H:%M:%S").ok()?;

    let time = chrono_tz::America::Chicago
        .from_local_datetime(&naive_time)
        .single()?;

    Some(time.timestamp())
}

fn timestamp_from_str_u64(timestamp: &str) -> Option<u64> {
    let naive_time = chrono::NaiveDateTime::parse_from_str(&timestamp, "%Y-%m-%dT%H:%M:%S").ok()?;

    let time = chrono_tz::America::Chicago
        .from_local_datetime(&naive_time)
        .single()?;

    Some(time.timestamp() as u64)
}

fn english_only_translations(text: String) -> TranslatedString {
    TranslatedString {
            translation: vec![Translation {
            text: text,
            language: Some("en_US".to_string()),
        }]
    }
}

pub async fn train_feed(
    client: &reqwest::Client,
    key: &str,
    trips: &str,
    gtfs: &gtfs_structures::Gtfs,
) -> Result<ChicagoResults, Box<dyn std::error::Error + Sync + Send>> {
    let mut trip_data_raw = csv::Reader::from_reader(trips.as_bytes());
    let trip_data = trip_data_raw.records();

    let mut run_ids_to_trip_ids: HashMap<u16, Vec<String>> = HashMap::new();

    for record in trip_data {
        let record = record?;
        //only identify train data
        if record[8].contains('R') {
            let run = record[8].replace("R", "");

            let run_number = run.parse::<u16>().unwrap();

            run_ids_to_trip_ids
                .entry(run_number)
                .or_default()
                .push(record[2].to_string());
        }
    }

    //println!("Sending req");

    let response = client
        .get("https://www.transitchicago.com/api/1.0/ttpositions.aspx")
        .query(&[
            ("key", &key),
            ("rt", &alltrainlines),
            ("outputType", &"JSON"),
        ])
        .send()
        .await;

    //println!("Got response");

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
    let ttpositions = json_output.ctatt;

    //Vec<TTPosTrain> or TTPosTrain

    let mut train_positions: Vec<FeedEntity> = vec![];
    let mut trip_updates: Vec<FeedEntity> = vec![];
    let mut alerts: Vec<FeedEntity> = vec![];

    for train_line_group in ttpositions.route {
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

            let current_date_to_search_chicago = current_chicago_time();

            let current_date = current_date_to_search_chicago.date_naive();

            let search_dates_timeline = [
                current_date.pred_opt().unwrap(),
                current_date,
                current_date.succ_opt().unwrap(),
            ];

            let trip_ids_to_check = run_ids_to_trip_ids
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<String>>();

            let trip_raw_list = trip_ids_to_check
                .iter()
                .map(|trip_id| gtfs.trips.get(trip_id))
                .filter(|x| x.is_some())
                .map(|x| x.unwrap());

            let service_ids_to_check = trip_raw_list
                .map(|x| x.service_id.clone())
                .collect::<HashSet<String>>();

            let mut service_ids_to_valid_dates: HashMap<String, Vec<chrono::NaiveDate>> =
                HashMap::new();

            for service_id in service_ids_to_check {
                let calendar_dates_for_service_id = gtfs.calendar_dates.get(&service_id);

                let calendar_for_service_id = gtfs.calendar.get(&service_id);

                for date_to_check in search_dates_timeline {
                    let mut allowed_date = false;

                    match calendar_for_service_id {
                        Some(calendar) => {
                            let in_date_range = date_to_check <= calendar.end_date
                                && date_to_check >= calendar.start_date;

                            let match_weekday = match date_to_check.weekday() {
                                chrono::Weekday::Mon => calendar.monday,
                                chrono::Weekday::Tue => calendar.tuesday,
                                chrono::Weekday::Wed => calendar.wednesday,
                                chrono::Weekday::Thu => calendar.thursday,
                                chrono::Weekday::Fri => calendar.friday,
                                chrono::Weekday::Sat => calendar.saturday,
                                chrono::Weekday::Sun => calendar.sunday,
                            };

                            let matching_date = in_date_range && match_weekday;

                            if matching_date {
                                allowed_date = true;
                            }
                        }
                        _ => {}
                    }

                    match calendar_dates_for_service_id {
                        Some(calendar_dates) => {
                            let date_find = calendar_dates.iter().find(|x| x.date == date_to_check);

                            if let Some(date_find) = date_find {
                                match date_find.exception_type {
                                    gtfs_structures::Exception::Added => {
                                        allowed_date = true;
                                    }
                                    gtfs_structures::Exception::Deleted => {
                                        allowed_date = false;
                                    }
                                }
                            }
                        }
                        None => {}
                    }

                    if allowed_date {
                        service_ids_to_valid_dates
                            .entry(service_id.clone())
                            .or_default()
                            .push(date_to_check);
                    }
                }
            }

            for train in &train_data_vec {
                let lat = train.lat.parse::<f32>();
                let lon = train.lon.parse::<f32>();

                let train_run_id = train.rn.parse::<u16>().unwrap();

                let possible_trip_ids = run_ids_to_trip_ids.get(&train_run_id);

                let timestamp = match timestamp_from_str(&train.prdt) {
                    Some(ts) => Some(ts as u64),
                    None => Some(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_secs(),
                    ),
                };

                let mut ranking_trips = vec![];

                if let Some(possible_trip_ids) = possible_trip_ids {
                    for possible_trip_id in possible_trip_ids {
                        let check_trip = gtfs.trips.get(possible_trip_id);

                        if let Some(check_trip) = check_trip {
                            let valid_service_dates =
                                service_ids_to_valid_dates.get(check_trip.service_id.as_str());

                            if let Some(valid_service_dates) = valid_service_dates {
                                for valid_service_date in valid_service_dates {
                                    let midnight_chicago_from_naive_date =
                                        midnight_chicago_from_naive_date(*valid_service_date);

                                    let trip_start_time = midnight_chicago_from_naive_date
                                        + chrono::Duration::seconds(
                                            check_trip.stop_times[0].departure_time.unwrap().into(),
                                        );

                                    ranking_trips.push(InternalTripIdSearch {
                                        trip_id: possible_trip_id.clone(),
                                        service_date: *valid_service_date,
                                        trip_start_time: trip_start_time,
                                    });
                                }
                            }
                        }
                    }
                }

                ranking_trips.sort_by_key(|x| (current_chicago_time() - x.trip_start_time).abs());

                let train_trip_id = ranking_trips.first().map(|x| x.trip_id.clone());

                if train_trip_id.is_some() {
                    if let Ok(lat) = lat {
                        if let Ok(lon) = lon {
                            let response = client
                                .get("https://www.transitchicago.com/api/1.0/ttfollow.aspx")
                                .query(&[
                                    ("key", &key),
                                    ("runnumber", &train.rn.as_str()),
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
                            let json_output = serde_json::from_str::<TTFollow>(text.as_str())?;
                            let ttfollow = json_output.ctatt;

                            let mut stop_time_updates = Vec::new();

                            if let Some(eta) = ttfollow.eta {
                                for prediction in eta {
                                    let update = StopTimeUpdate {
                                        stop_sequence: None,
                                        stop_id: Some(prediction.stp_id.clone()),
                                        arrival: Some(StopTimeEvent {
                                            delay: None,
                                            time: timestamp_from_str(&prediction.arrt),
                                            uncertainty: None,
                                            scheduled_time: None,
                                        }),
                                        departure: None,
                                        departure_occupancy_status: None,
                                        schedule_relationship: None,
                                        stop_time_properties: None, // Most of these fields are still experimental and not part of the rust library yet
                                                                    // stop_time_properties: Some(StopTimeProperties {
                                                                    //     assigned_stop_id: None,
                                                                    //     stop_headsign: &prediction.dest_nm,
                                                                    //     drop_off_type: None,
                                                                    //     pickup_type: None
                                                                    // })
                                    };

                                    stop_time_updates.push(update);
                                }
                            } else {
                                stop_time_updates.push(StopTimeUpdate {
                                    stop_sequence: None,
                                    stop_id: Some(train.next_stp_id.clone()),
                                    arrival: Some(StopTimeEvent {
                                        delay: None,
                                        time: timestamp_from_str(&train.arrt),
                                        uncertainty: None,

                                        scheduled_time: None,
                                    }),
                                    departure: None,
                                    departure_occupancy_status: None,
                                    schedule_relationship: None,
                                    stop_time_properties: None,
                                });
                            }

                            let pos_entity: FeedEntity = FeedEntity {
                                id: train.rn.clone(),
                                stop: None,
                                trip_modifications: None,
                                is_deleted: None,
                                trip_update: None,
                                vehicle: Some(gtfs_realtime::VehiclePosition {
                                    trip: Some(gtfs_realtime::TripDescriptor {
                                        modified_trip: None,
                                        trip_id: train_trip_id.clone(),
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
                                    timestamp: timestamp_from_str(&ttfollow.tmst).map(|i| i as u64),
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
                                        trip_id: train_trip_id.clone(),
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
                                    stop_time_update: stop_time_updates,
                                    timestamp: timestamp_from_str(&ttfollow.tmst).map(|i| i as u64),
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

    // Query CTA Customer Alerts API for alerts
    let response = client
        .get("https://www.transitchicago.com/api/1.0/alerts.aspx")
        .query(&[
            ("outputType", &"JSON"),
            ("activeonly", &"true"),
        ])
        .send()
        .await;

    if let Err(response) = &response {
        println!(
            "{color_magenta}{:#?}{color_reset}",
            response.url().unwrap().as_str()
        );
    }

    let response = response?;
    let text = response.text().await?;
    let json_output = serde_json::from_str::<CTAAlerts>(text.as_str())?;
    let alerts_data = json_output.cta_alerts.alert;

    for alert in alerts_data {
        let active_period: Vec<TimeRange> = vec![
            TimeRange {
                start: match alert.event_start {
                    Some(start) => timestamp_from_str_u64(&start),
                    None => None
                },
                end: match alert.event_end {
                    Some(end) => timestamp_from_str_u64(&end),
                    None => None
                },
            }
        ];

        let mut informed_entity: Vec<EntitySelector> = Vec::new();
        for impacted_service in alert.impacted_service.service {
            if impacted_service.service_type == "T" {
                informed_entity.push(EntitySelector {
                    stop_id: Some(impacted_service.service_id),
                    ..EntitySelector::default()                 
                });
            } else {
                informed_entity.push(EntitySelector {
                    route_id: Some(impacted_service.service_id),
                    ..EntitySelector::default()
                });
            }
        }

        let effect = match alert.impact.as_str() {
            "Bus Stop Note" => Effect::StopMoved,
            "Bus Stop Relocation" => Effect::StopMoved,
            "Elevator Status" => Effect::AccessibilityIssue,
            "Normal Service*" => Effect::NoEffect,
            "Planned Reroute" => Effect::Detour,
            "Planned Work" => Effect::ModifiedService,
            "Service Change" => Effect::ModifiedService,
            "Special Note" => Effect::UnknownEffect,
            _ => Effect::UnknownEffect,
            // TODO - The full list of allowed impacts is not documented
            // This list was found by looking through the feed
        };

        let cause = Cause::UnknownCause;
        let effect_detail = alert.impact;

        let url = alert.alert_url.cdata_section;
        let header_text = alert.headline;
        let description_text = alert.short_description;

        let severity_level = match alert.severity_css.as_str() {
            "normal" => SeverityLevel::Info,
            "planned" => SeverityLevel::Info,
            "minor" => SeverityLevel::Warning,
            "major" => SeverityLevel::Severe,
            _ => SeverityLevel::UnknownSeverity,
        };

        let wrapped_description_text = match description_text {
            Some(desc) => Some(english_only_translations(desc)),
            None => None,
        };

        alerts.push(FeedEntity {
            id: alert.guid, 
            is_deleted: None,
            trip_update: None,
            vehicle: None,
            alert: Some(Alert {
                active_period: active_period,
                informed_entity: informed_entity,
                cause: Some(cause.into()),
                effect: Some(effect.into()),
                url: Some(english_only_translations(url)),
                header_text: Some(english_only_translations(header_text.clone())),
                description_text: wrapped_description_text.clone(),
                tts_header_text: Some(english_only_translations(header_text.clone())),
                tts_description_text: wrapped_description_text.clone(),
                severity_level: Some(severity_level.into()),
                image: None,
                image_alternative_text: None,
                cause_detail: None,
                effect_detail: Some(english_only_translations(effect_detail)),
            }),
            shape: None,
            stop: None,
            trip_modifications: None
        });
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
                feed_version: None,
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
                feed_version: None,
            },
        },
        alerts: gtfs_realtime::FeedMessage {
            entity: alerts,
            header: gtfs_realtime::FeedHeader {
                timestamp: Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs(),
                ),
                gtfs_realtime_version: String::from("2.0"),
                incrementality: None,
                feed_version: None,
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

        println!("Reading gtfs data");
        let gtfs_data = gtfs_structures::Gtfs::new("static/").unwrap();
        println!("Finished reading gtfs data");

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
            &gtfs_data,
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
