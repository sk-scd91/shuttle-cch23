use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant
};
use axum::{
    extract::{ Json, Path, State },
    http::StatusCode,
    routing::{ get, post },
    Router,
};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Default)]
struct StringTimes {
    store: BTreeMap<String, Instant>,
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

pub fn timekeeper_router() -> Router {
    let state = Arc::new(Mutex::new(StringTimes::default()));
    Router::new()
        .route("/save/:s", post(save_time))
        .route("/load/:s", get(load_time))
        .route("/ulids", post(convert_ulids))
        .with_state(state)
}