use axum::{
    extract::Json,
    routing::post,
    Router,
};
use serde::Serialize;

#[derive(Default, Serialize)]
struct ElfCounts {
    elf: u64,
    #[serde(rename="elf on a shelf")]
    elf_on_shelf: u64,
    #[serde(rename="shelf with no elf on it")]
    shelf_without_elf: u64
}

async fn count_elves(phrase: String) -> Json<ElfCounts> {
    let phrase = phrase.to_ascii_lowercase();
    let mut elf_counts = ElfCounts::default();
    elf_counts.elf = phrase.matches("elf").count() as u64;
    // Could use Regex, but this feels less like cheating.
    for (i, _) in phrase.match_indices("shelf") {
        let elf_substr_start = i.saturating_sub("elf on a ".len());
        let shelf_prefix_start = elf_substr_start.saturating_sub(2);
        if &phrase[elf_substr_start..i] == "elf on a " && &phrase[shelf_prefix_start..elf_substr_start] != "sh" {
            elf_counts.elf_on_shelf += 1;
        } else {
            elf_counts.shelf_without_elf += 1;
        }
    }
    Json(elf_counts)
}

pub fn elf_router() -> Router {
    Router::new().route("/", post(count_elves))
}