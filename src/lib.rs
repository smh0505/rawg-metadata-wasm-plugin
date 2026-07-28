//! WASM `MetadataProviderPlugin` for RAWG. Provides description/release date/genres (no
//! cover/background art - that's SteamGridDB's job, see `sgdb-metadata-wasm-plugin`), same
//! text-only scoping as `igdb-metadata-wasm-plugin`.
//!
//! Requires an API key, set via this plugin's `settingsSchema`-declared `api_key` setting
//! (see `plugin.json`) - read back here through `host::settings-get`, namespaced by the host
//! per plugin id so it can never collide with another plugin's settings.
//!
//! RAWG's search ranks by its own relevance score, not popularity/exactness - a low-quality/
//! unrelated listing can genuinely outrank the real game (confirmed for real: searching "A
//! Dance of Fire and Ice" put an obscure 2014 itch.io prototype ahead of the actual 2019
//! release). Only listings whose `name` is an exact case-insensitive match to the query are
//! ever surfaced as candidates at all - a query with no exact-name match returns zero
//! candidates (left blank by the host) rather than guessing at a fuzzy one. When more than one
//! listing shares the exact same name (a real possibility - remasters/reissues/re-releases),
//! `label` appends the release year so the host's candidate picker can actually tell them
//! apart.

#[allow(warnings)]
mod bindings;

use bindings::exports::gamelib::plugin::metadata_plugin::{Guest, MetadataCandidate, MetadataResult};
use bindings::gamelib::plugin::host;

struct RawgPlugin;

#[derive(serde::Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(serde::Deserialize)]
struct SearchResult {
    id: u64,
    name: String,
    released: Option<String>,
    background_image: Option<String>,
}

#[derive(serde::Deserialize)]
struct DetailResponse {
    released: Option<String>,
    description_raw: Option<String>,
    genres: Vec<RawgGenre>,
}

#[derive(serde::Deserialize)]
struct RawgGenre {
    name: String,
}

fn api_key() -> Result<String, String> {
    host::settings_get("api_key").ok_or_else(|| "RAWG API Key not set - configure it in Settings.".to_string())
}

fn candidate_label(result: &SearchResult) -> String {
    match &result.released {
        Some(released) => format!("{} ({})", result.name, released),
        None => format!("{} (release date unknown)", result.name),
    }
}

impl Guest for RawgPlugin {
    fn search_candidates(title: String) -> Result<Vec<MetadataCandidate>, String> {
        let key = api_key()?;
        let url = format!(
            "https://api.rawg.io/api/games?key={}&search={}&page_size=10",
            urlencoding::encode(&key),
            urlencoding::encode(&title)
        );
        let body = host::http_get(&url)?;
        let search: SearchResponse = serde_json::from_str(&body).map_err(|e| e.to_string())?;

        Ok(search
            .results
            .into_iter()
            .filter(|r| r.name.eq_ignore_ascii_case(&title))
            .map(|r| MetadataCandidate {
                id: r.id.to_string(),
                label: candidate_label(&r),
                image_url: r.background_image.clone(),
            })
            .collect())
    }

    fn fetch_metadata_by_id(id: String) -> Result<Option<MetadataResult>, String> {
        let key = api_key()?;
        let url = format!("https://api.rawg.io/api/games/{}?key={}", id, urlencoding::encode(&key));
        let body = host::http_get(&url)?;
        let detail: DetailResponse = serde_json::from_str(&body).map_err(|e| e.to_string())?;

        Ok(Some(MetadataResult {
            description: detail.description_raw.filter(|d| !d.is_empty()),
            release_date: detail.released,
            genres: detail.genres.into_iter().map(|g| g.name).collect(),
            cover_art_url: None,
            background_art_url: None,
        }))
    }
}

bindings::export!(RawgPlugin with_types_in bindings);
