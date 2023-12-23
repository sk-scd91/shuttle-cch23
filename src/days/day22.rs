use std::{
    collections::{HashMap,  VecDeque},
    ops::BitXor,
};

use axum::{
    routing::post,
    Router,
};

async fn find_only_integer(input: String) -> String {
    // Since x XOR x = 0, we can find the unique single integer by xoring the entire input.
    let input = input.lines()
        .filter_map(|l| l.parse::<u64>().ok())
        .fold(0u64, BitXor::bitxor);

    // Repeatedly write the present emoji to the output string.
    let present = '🎁';
    std::iter::repeat(present)
        .take(input as usize)
        .collect::<String>()
}

#[derive(Clone, Copy, Default)]
struct SpaceCoord(i32, i32, i32);

fn distance(SpaceCoord(ax, ay, az): SpaceCoord, SpaceCoord(bx, by, bz): SpaceCoord) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    let dz = az - bz;

    f32::sqrt((dx * dx + dy * dy + dz * dz) as f32)
}

// Find shortest teleportation path using Breadth First Search algorithm.
fn find_shortest_path_and_distance(
    paths: &HashMap::<usize, Vec<(usize, f32)>>,
    dest: usize
) -> Option<(usize, f32)> {

    // A map of shortest iterations and distances, indexed by key.
    let mut visiting = HashMap::new();
    visiting.insert(0usize, (0usize, 0f32));
    
    // A queue to run a BFS on each adjacent star.
    let mut visited = VecDeque::new();
    visited.push_back(0usize);

    while let Some(index) = visited.pop_front() {
        // If we found our destination, and exausted all closer paths, return with the count and aggegate distance.
        if index == dest {
            return visiting.get(&index).copied();
        }

        // Otherwise, find adjacent stars not yet found.
        let &(iteration, src_dist) = visiting.get(&index).unwrap();
        for &(next_dest, dest_dist) in paths.get(&index).unwrap() {
            use std::collections::hash_map::Entry;
            let next_dist = src_dist + dest_dist;
            match visiting.entry(next_dest) {
                // If not found, insert into the visited map and enqueue.
                Entry::Vacant(v) => {
                    v.insert((iteration + 1, next_dist));
                    visited.push_back(next_dest);
                },
                Entry::Occupied(mut o) => {
                    // Only mutate if there is a shorter distance for the next iteration.
                    if iteration + 1 == o.get().0 && next_dist < o.get().1 {
                        *o.get_mut() = (iteration + 1, next_dist);
                    }
                }
            };
        }
    }

    None
}

async fn find_rocket_path(input: String) -> Result<String, String> {
    let mut lines = input.lines();

    // First, get the star coords.
    let star_count = match lines.next() {
        Some(v) => v,
        None => { return Err("No input".into());}
    };
    let star_count = star_count.parse::<usize>()
        .map_err(|e| format!("Parsing error: {}", e))?;

    let mut star_coords = vec![];
    for l in std::iter::repeat_with(|| lines.next()).take(star_count) {
        let coord_str = match l {
            Some(v) => v,
            None => { return Err("No input".into());}
        };

        let coord_vals = coord_str.split_ascii_whitespace()
            .filter_map(|c| c.trim().parse::<i32>().ok())
            .collect::<Vec<i32>>();

        if coord_vals.len() != 3 {
            return Err(format!("Failed parsing coord string: {}", coord_str));
        }

        star_coords.push(SpaceCoord(coord_vals[0], coord_vals[1], coord_vals[2]));
    }

    // Then, get the teleportation paths.
    let portal_count = match lines.next() {
        Some(v) => v,
        None => { return Err("No input".into());}
    };
    let portal_count = portal_count.parse::<usize>()
        .map_err(|e| format!("Parsing error: {}", e))?;

    let mut portal_paths = HashMap::<usize, Vec<(usize, f32)>>::new();
    for l in std::iter::repeat_with(|| lines.next()).take(portal_count) {
        let portal_str = match l {
            Some(v) => v,
            None => { return Err("No input".into());}
        };

        let portal_vals = portal_str.split_ascii_whitespace()
            .filter_map(|c| c.trim().parse::<usize>().ok())
            .collect::<Vec<usize>>();

        if portal_vals.len() != 2 {
            return Err(format!("Failed parsing coord string: {}", portal_str));
        }

        // Calculate distance.
        let dist = distance(star_coords[portal_vals[0]], star_coords[portal_vals[1]]);
        // Create bi-directional entry.
        portal_paths.entry(portal_vals[0])
            .or_insert_with(Vec::new)
            .push((portal_vals[1], dist));
        portal_paths.entry(portal_vals[1])
            .or_insert_with(Vec::new)
            .push((portal_vals[0], dist));
    }

    drop(lines); // Now we have all input.

    // First, get the shortest portal path.
    let (shortest_len, dist) = match find_shortest_path_and_distance(&portal_paths, star_count.saturating_sub(1)){
        Some(v) => v,
        None => { return Err("Could not find path.".into());}
    };

    // Return count and distance with 3 decimal precision.
    Ok(format!("{} {:.3}", shortest_len, dist))
}

pub fn final_router() -> Router {
    Router::new().route("/integers", post(find_only_integer))
        .route("/rocket", post(find_rocket_path))
}