# rawg-metadata-wasm-plugin

A `MetadataProviderPlugin` for [Concourse](https://github.com/smh0505/Concourse) implemented as
a WASM component. Fetches description/release date/genres from
[RAWG](https://rawg.io/apidocs) by title search - no cover/background art, that's
SteamGridDB's job (see `sgdb-metadata-wasm-plugin`), same text-only scoping as
`igdb-metadata-wasm-plugin`.

This is a real, separate repo on purpose - same reasoning as `steam-source-wasm-plugin`: a
plugin whose source lives inside the host app's own repo doesn't genuinely exercise the
"install arbitrary third-party code" model the WASM plugin system is for.

RAWG's search endpoint doesn't return the full description text - only the detail endpoint
does - so a match takes two requests: search to resolve an id (also carries released date and
genres, no second call needed for those), then a detail fetch for the description.

Requires a RAWG API key (get one at [rawg.io/apidocs](https://rawg.io/apidocs)) - set it in
Concourse's Settings under this plugin's row (rendered from `plugin.json`'s `settingsSchema`, a
generic form the host builds for any WASM plugin that declares one; no custom UI code needed on
either side).

## Permissions

Declares `httpScopes: ["api.rawg.io"]` (Milestone 13 URL allowlisting).

## Building

```sh
rustup target add wasm32-wasip1   # once
cargo install cargo-component     # once
cargo component build
```

Output: `target/wasm32-wasip1/debug/rawg_metadata_wasm_plugin.wasm`.

## Installing into a running Concourse

Either build locally (above) or grab the prebuilt `.wasm` + `plugin.json` from this repo's
[Releases](https://github.com/smh0505/rawg-metadata-wasm-plugin/releases) - CI (`.github/workflows/publish.yml`) publishes a new release
automatically whenever `plugin.json`'s `version` is bumped on `main`. Concourse's Settings ->
Metadata Provider tab -> Add Plugin also accepts a Release's `plugin.json` URL directly
(metadata-kind plugins install by URL, same as source plugins) - the latest one always lives
at:

```
https://github.com/smh0505/rawg-metadata-wasm-plugin/releases/latest/download/plugin.json
```

Copy the compiled `.wasm` and `plugin.json` into
`<app data dir>/wasm-plugins/metadata/rawg-wasm/` (Windows:
`%APPDATA%\com.bloppy.concourse\wasm-plugins\metadata\rawg-wasm\`). It'll show up in Settings'
Plugins panel under the Metadata Provider tab next time the app starts, as **RAWG**.

## Versioning

Plain SemVer (`Cargo.toml` + `plugin.json`'s `version`), independent of Concourse's own
milestone-tracked version - patch for fixes, minor for backward-compatible new capabilities,
major for breaking manifest/WIT interface changes. Full convention:
[`.claude/CLAUDE.md`](https://github.com/smh0505/Concourse/blob/main/.claude/CLAUDE.md) (Plugin Versioning) in the main [Concourse](https://github.com/smh0505/Concourse) repo.
