use axum::{
    extract::Path,
    http::StatusCode,
    routing::get,
    Router,
};
use photon_geocoding::{
    filter::ReverseFilter,
    LatLon,
    PhotonApiClient
};
use s2::{
    cell::Cell,
    cellid::CellID,
};

fn degrees_to_dms(angle: f64, is_latitude: bool) -> (u64, u64, f64, char) {
    // Readjust latitude in case it wraps around the pole.
    let angle = if is_latitude && angle.abs() > 90.0 {
        180.0 - angle
    } else {
        angle
    };
    let degrees = angle.abs().trunc() as u64;
    let ms = angle.abs().fract() * 60.0;
    let minutes = ms.trunc() as u64;
    let seconds = ms.fract() * 60.0;
    let direction = match (is_latitude, angle > 0.0) {
        (false, false) => 'W',
        (false, true) => 'E',
        (true, false) => 'S',
        (true, true) => 'N',
    };
    (degrees, minutes, seconds, direction)
}

async fn get_coords_for_hilbert(Path(code): Path<String>) -> Result<String, (StatusCode, String)> {
    let code_num = u64::from_str_radix(&code, 2)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Cannot parse string: {}", e)))?;

    let s2_cell = Cell::from(CellID(code_num));
    let (lat, lon) = {
        let point = s2_cell.center();
        (point.latitude().deg(), point.longitude().deg())
    };

    // Convert degrees to DMS format.
    let (lat_deg, lat_min, lat_sec, lat_dir) = degrees_to_dms(lat, true);
    let (lon_deg, lon_min, lon_sec, lon_dir) = degrees_to_dms(lon, false);
    
    Ok(
        format!("{}°{}'{:.3}''{} {}°{}'{:.3}''{}", lat_deg, lat_min, lat_sec, lat_dir, lon_deg, lon_min, lon_sec, lon_dir)
    )
}

async fn get_country_from_hilbert(Path(code): Path<String>) -> Result<String, (StatusCode, String)> {
    let code_num = u64::from_str_radix(&code, 2)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Cannot parse string: {}", e)))?;

    let s2_cell = Cell::from(CellID(code_num));
    let (lat, lon) = {
        let point = s2_cell.center();
        (point.latitude().deg(), point.longitude().deg())
    };

    tracing::info!("Parsed coordinates: {lat}, {lon}"); // To verify coordinates in other sources.

    // Use https://photon.komoot.io/ for country lookup.
    // Data provided by https://openstreetmap.org
    let client = PhotonApiClient::default();
    let request = tokio::task::spawn_blocking(move || client.reverse_search(
            LatLon::new(lat, lon), 
            Some(
                ReverseFilter::new().language("EN")
                    .radius(10)
                    .limit(1)
            )
        ).map_err(|e| (StatusCode::BAD_GATEWAY, format!("Unable to process Photon data: {}", e)))
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", e)))??;

    tracing::info!("Found results: {}", request.len());

    if let Some(Some(country)) = request.into_iter().next().map(|f| f.country.clone()) {
        Ok(country)
    } else {
        Err((StatusCode::NOT_FOUND, "Unable to find country data.".into()))
    }
}

pub fn world_coord_router() -> Router {
    Router::new().route("/coords/:bin", get(get_coords_for_hilbert))
        .route("/country/:bin", get(get_country_from_hilbert))
}