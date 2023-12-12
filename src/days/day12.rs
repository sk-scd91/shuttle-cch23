use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use axum::{
    extract::{ Json, Path, State },
    http::StatusCode,
    routing::{ get, post },
    Router,
};
use chrono::{
    Datelike,
    NaiveDate,
    NaiveDateTime,
};
use serde::Serialize;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Default)]
struct StringTimes {
    store: BTreeMap<String, Instant>,
}

#[derive(Serialize, Default)]
struct UlidStats {
    #[serde(rename="christmas eve")]
    christmas_eve: usize,
    weekday: usize,
    #[serde(rename="in the future")]
    in_the_future: usize,
    #[serde(rename="LSB is 1")]
    lsb_is_1: usize
}

async fn save_time(
    State(string_times): State<Arc<Mutex<StringTimes>>>,
    Path(s): Path<String>
) {
    let mut string_times = string_times.lock()
        .expect("StringTimes lock should not be poisoned.");
    let _ = string_times.store.insert(s.clone(), Instant::now());
}

async fn load_time(
    State(string_times): State<Arc<Mutex<StringTimes>>>,
    Path(s): Path<String>
) -> Result<String, (StatusCode, String)> {
    let last_instant = {
        let string_times = string_times.lock()
            .expect("StringTimes lock should not be poisoned.");
        string_times.store.get(&s).map(Instant::to_owned)
    }.ok_or_else(||
        (
            StatusCode::NOT_FOUND,
            format!("Cannot find time for string: {}", &s),
        )
    )?;
    
    let elapsed = last_instant.elapsed();
    Ok(elapsed.as_secs().to_string())
}

async fn convert_ulids(Json(ulids): Json<Vec<String>>) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let mut uuids = Vec::new();

    for encoded in ulids.iter().rev() {
        let ulid = Ulid::from_string(encoded)
            .map_err(
                |e| (StatusCode::BAD_REQUEST, format!("Unable to parse Ulid: {}", e))
            )?;
        let uuid: Uuid = ulid.into();
        uuids.push(uuid.to_string());
    }

    Ok(Json(uuids))
}

async fn ulid_stats(
    Path(weekday): Path<u64>,
    Json(ulids): Json<Vec<String>>,
) -> Result<Json<UlidStats>, (StatusCode, String)> {
    let now_ulid = Ulid::new();
    let mut stats = UlidStats::default();

    let chrismas_eve = NaiveDate::from_ymd_opt(2023, 12, 24).unwrap();

    for encoded in ulids.iter().rev() {
        let ulid = Ulid::from_string(encoded)
            .map_err(
                |e| (StatusCode::BAD_REQUEST, format!("Unable to parse Ulid: {}", e))
            )?;
        
        // Get the date from the Ulid.
        let ulid_time = NaiveDateTime::from_timestamp_millis(ulid.timestamp_ms() as i64)
            .unwrap();
        let ulid_date: NaiveDate = ulid_time.into();

        // Compare ordinal numbers for days of the year. Adjust for leap years.
        if chrismas_eve.ordinal0() + ulid_date.leap_year() as u32 == ulid_date.ordinal0() {
            stats.christmas_eve += 1;
        }
        // Compare weekdays, starting with Monday as 0.
        if ulid_date.weekday().num_days_from_monday() == weekday as u32 {
            stats.weekday += 1;
        }
        // ms units for both are based on SystemTime in UTC.
        if ulid.timestamp_ms() > now_ulid.timestamp_ms() {
            stats.in_the_future += 1;
        }
        // Get the lsb of the decoded id.
        if ulid.0 & 1u128 == 1u128 {
            stats.lsb_is_1 += 1;
        }
    }

    Ok(Json(stats))
}

pub fn timekeeper_router() -> Router {
    let state = Arc::new(Mutex::new(StringTimes::default()));
    Router::new()
        .route("/save/:s", post(save_time))
        .route("/load/:s", get(load_time))
        .route("/ulids", post(convert_ulids))
        .route("/ulids/:weekday", post(ulid_stats))
        .with_state(state)
}