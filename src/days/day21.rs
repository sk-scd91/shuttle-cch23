use axum::{
    extract::Path,
    http::StatusCode,
    routing::get,
    Router,
};
use s2::{
    cell::Cell,
    cellid::CellID,
};

// Converts a 64 bit hilbert curve code with a trailing 1 bit to two 32 bit coordinates.
// See: http://s2geometry.io/devguide/s2cell_hierarchy#hilbert-curve
/* // Unable to use due to time limit. Keeping here for recognition.
fn hilbert_to_leaf_coords(hilbert_code: u64) -> (u32, u32) {
    let mut remaining = hilbert_code & hilbert_code.wrapping_sub(1); //remove last one, if exists.
    let (mut coord_x, mut coord_y, mut rot, mut flip) = (0u32, 0u32, 0u32, 0u32);

    // Shortcut: Map every two bits to match this pattern
    // _|_0_|_1_
    // 0|0,0|1,0
    // 1|0,1|1,1
    const EVERY_EVEN_BIT: u64 = 0x5555_5555_5555_5555;
    remaining ^= (remaining >> 1) & EVERY_EVEN_BIT;

    while remaining != 0 {
        let quad = (remaining >> (u64::BITS - 2)) as u32;

        // Match odd bits to x and even bits to y, or the other way around if rotated.
        // Complement the bit if rotated right.
        coord_x <<= 1;
        coord_y <<= 1;
        coord_x |= ((quad >> (rot ^ 1)) ^ flip) & 1;
        coord_y |= ((quad >> rot) ^ flip) & 1;

        // Rotate on cells 0 or 3, and flip on 3.
        rot ^= !(quad ^ (quad >> 1)) & 1;
        flip ^= quad & (quad >> 1) & 1;

        // Shift to the next remaining two bits.
        remaining <<= 2;
    }

    // Adjust coordinates.
    let shift = (hilbert_code.trailing_zeros() as u32 + 1) >> 1;
    (coord_x.wrapping_shl(shift), coord_y.wrapping_shl(shift))
}

*/

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

pub fn world_coord_router() -> Router {
    Router::new().route("/coords/:bin", get(get_coords_for_hilbert))
}