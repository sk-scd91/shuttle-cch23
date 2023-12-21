use std::collections::{HashMap, HashSet};

use axum::{
    body::Bytes,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    middleware::{from_fn, Next},
    response::IntoResponse,
    routing::post,
    Router
};
use flate2::read::ZlibDecoder;

use tar::Archive;

struct TreeEntry(u32, String, String);

async fn tar_only_middleware<T>(request: Request<T>, next: Next<T>) -> Result<impl IntoResponse, StatusCode> {
    let content_type = request.headers().get(CONTENT_TYPE);

    // If content header matches, or is missing.
    if content_type.map(|h| h.to_str().unwrap_or("") == "application/x-tar").unwrap_or(true) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNSUPPORTED_MEDIA_TYPE)
    }
}

async fn file_count_in_tar(tar_data: Bytes) -> Result<String, (StatusCode, String)> {
    let mut tar_archive = Archive::new(tar_data.as_ref());
    tar_archive.entries()
        .map(|es| es.count().to_string())
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Unable to parse tar: {}", e)))
}

async fn file_size_in_tar(tar_data: Bytes) -> Result<String, (StatusCode, String)> {
    let mut tar_archive = Archive::new(tar_data.as_ref());
    tar_archive.entries()
        .map(
            |es| {
                es.filter_map(|e| e.ok())
                    .map(|e| e.size())
                    .sum::<u64>()
                    .to_string()
            }
        ).map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Unable to parse tar: {}", e)))
}

#[axum::debug_handler]
async fn find_cookie(tar_data: Bytes) -> Result<String, (StatusCode, String)> {
    let mut tar_archive = Archive::new(tar_data.as_ref());
    
    let mut objs = HashMap::<String, &[u8]>::new();
    let mut ref_hash = String::new();

    // First, get all of the files, and put them in a map.
    // Assumes uncompressed tar files, which are already allocated in Bytes.
    let entries = tar_archive.entries()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Unable to get archive entries: {}", e)))?;
    for entry_result in entries {
        match entry_result {
            Ok(entry) => {
                let path = entry.path().unwrap();
                // Capture object files, with at least a SHA-1 HASH.
                if path.starts_with(".git/objects/") && path.file_name().unwrap().len() == 38 {
                    let new_path = {
                        let mut components = path.components();
                        let fname = components.next_back().unwrap();
                        format!(
                            "{}{}",
                            components.next_back().unwrap().as_os_str().to_string_lossy(),
                            fname.as_os_str().to_string_lossy()
                        )
                    };
                    //println!("Hash: {}", new_path);
                    let position = entry.raw_file_position() as usize;
                    let tar_slice = &tar_data[position..position + entry.size() as usize];
                    objs.insert(new_path, tar_slice);
                }
                else if path.to_str() == Some(".git/refs/heads/christmas") {
                    let position = entry.raw_file_position() as usize;
                    let tar_slice = &tar_data[position..position + entry.size() as usize - 1];
                    ref_hash.push_str(&String::from_utf8_lossy(&tar_slice));
                    //println!("Ref: {}", ref_hash);
                }
            },
            Err(e) => { return Err((StatusCode::UNPROCESSABLE_ENTITY, format!("Unable to get entry: {}", e))); }
        }
    }

    let objs = objs; // Make mutable
    let mut next_hash = Some(ref_hash);
    let mut found_set = HashSet::<String>::new();

    // Search through all commits to see which one has the right answer.
    tokio::task::block_in_place(|| {
        while let Some(commit) = next_hash.clone().map(|r| objs.get(&r)).flatten() {
            let commit = uncompress_obj(*commit)
                .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Unable to decompress object: {}", e)))?;
            if !commit.starts_with(b"commit") {
                return Err((StatusCode::UNPROCESSABLE_ENTITY, "Not a commit entry.".into()));
            }
            // Split null-terminated part of string.
            let commit_string = commit.splitn(2, |&u| u == 0)
                .nth(1)
                .unwrap();

            let commit_string = String::from_utf8_lossy(commit_string);
            //println!("Commit: {}", &commit_string);

            let tree_hash = commit_string.lines()
                .filter_map(|l| l.strip_prefix("tree "))
                .map(str::to_string)
                .next()
                .ok_or_else(|| (StatusCode::UNPROCESSABLE_ENTITY, "Error finding tree in commit".into()))?;
            let author = commit_string.lines()
                .filter_map(|l| l.strip_prefix("author "))
                .filter_map(|l| l.split(" <").next()) // Extract all before email.
                .map(str::to_string)
                .next()
                .ok_or_else(|| (StatusCode::UNPROCESSABLE_ENTITY, "Error finding author in commit".into()))?;

            let result = find_file_with_cookie(&objs, &tree_hash, &found_set)
                .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;
            if let Err(other_set) = result {
                found_set.extend(other_set);
            } else {
                // Found the gifts. Return the author and commit hash.
                return Ok(format!("{} {}", author, next_hash.unwrap()))
            }

            next_hash = commit_string.lines()
                .filter_map(|l| l.strip_prefix("parent "))
                .map(str::to_string)
                .next();
        }
        Err((StatusCode::NOT_FOUND, "Cookie not found.".into()))
    })
}

// Uncompress a zlib file from a byte slice.
fn uncompress_obj(obj: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(obj);
    let mut buf: Vec<u8> = vec![];
    decoder.read_to_end(&mut buf)
        .and(Ok(buf))
}

fn find_file_with_cookie(
    objs: &HashMap<String, &[u8]>,
    tree_hash: &String,
    found: &HashSet<String>
) -> Result<Result<(), HashSet<String>>, String> {
    let mut local_found_set = HashSet::new();

    if let Some(tree) = objs.get(tree_hash) {
        let tree = uncompress_obj(*tree)
            .map_err(|e| format!("Unable to decompress tree: {}", e))?;
        if !tree.starts_with(b"tree ") {
            return Err("Cannot process tree.".into());
        }
        let entries = parse_git_tree(&tree)?;
        for TreeEntry(flags, filename, hash) in entries {
            //println!("{} {} {}", flags, filename, hash);
            if filename.trim() == "santa.txt" {
                if let Some(blob) = objs.get(&hash) {
                    let blob = uncompress_obj(*blob)
                        .map_err(|e| format!("Unable to decompress blob: {}", e))?;
                    if String::from_utf8_lossy(&blob).contains("COOKIE") {
                        return Ok(Ok(()));
                    }
                }
            }
            if !found.contains(&hash) && local_found_set.insert(hash.clone()) {
                // First, check if the branch is a possible directory.
                if flags == 40000 {
                    let result = find_file_with_cookie(objs, &hash, found)?;
                    if let Err(other_set) = result {
                        local_found_set.extend(other_set);
                    } else {
                        return Ok(Ok(()));
                    }
                } 
            }
        }
    } else {
       return Err("Cannot find tree.".into());
    }
    // Ran out of files in path.
    Ok(Err(local_found_set))
}

fn parse_git_tree(tree: &[u8]) -> Result<Vec<TreeEntry>, String> {
    let first_pos = match tree.iter().position(|&c| c == 0) {
        Some(p) => p,
        None => { return Err("No entries in tree.".into()) }
    };
    let mut tree_slice = &tree[first_pos + 1..];
    let mut result_vec = Vec::new();
    while let Some(p) = tree_slice.iter().position(|&c| c == 0) {
        let text_part = String::from_utf8_lossy(&tree_slice[..p]);
        let (flags, filename) = text_part.split_once(' ')
            .ok_or_else(|| format!("Cannot split string: {}", text_part))?;
        let flags = flags.to_string().parse::<u32>()
            .map_err(|e| format!("Cannot parse flags: {}", e))?;
        // Convert hash bytes into a hex string.
        let hash = tree_slice[p + 1..p + 21].iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<String>>()
            .join("");
        result_vec.push(TreeEntry(flags, filename.to_owned(), hash));
        tree_slice = &tree_slice[p + 21..];
    }
    Ok(result_vec)
}

pub fn archive_router() -> Router {
    Router::new().route("/archive_files", post(file_count_in_tar))
            .route("/archive_files_size", post(file_size_in_tar))
            .route("/cookie", post(find_cookie))
            .layer(from_fn(tar_only_middleware))
}