# AGENTS.md

## Repository Overview & Architecture

- **Stack**: Actix Web (Rust) backend + React/TypeScript (Vite + MUI) frontend in `ui/`.
- **Build Integration**: `cargo build` automatically executes `npm ci` and `npm run build` inside `ui/` via `build.rs`. Rust embeds `ui/dist/index.html` and `ui/src/seo/routes.json` at compile time using `include_str!`.
- **SPA Route Lockstep**: Adding or removing SPA page routes requires updating **3 places in lockstep**:
  1. Client routes in `ui/src/App.tsx`
  2. Server SPA routes in `src/startup.rs` and route metadata in `ui/src/seo/routes.json`
  3. Sitemap entries in `public/sitemap.xml`
- **SEO Architecture**: Route metadata source of truth is `ui/src/seo/routes.json`. Backend (`src/routes/home/mod.rs`) injects dynamic `<head>` SEO tags into `index.html` at the `<!--SEO-->` marker for server-side unfurls. Frontend (`ui/src/utils/seo.tsx`) reads `routes.json` for client-side navigation.

## Verification & Commands

Run verification commands in this order:

1. **Frontend**: `npm run build` (in `ui/` directory)
2. **Backend Tests**: `cargo test` (or `cargo test <test_name>` for a single test; `TEST_LOG=1 cargo test` for verbose tracing)
3. **Format Check**: `cargo fmt --check`
4. **Clippy Lint**: `cargo clippy -- -D warnings`

To apply formatting automatically: `cargo fmt`.

## Testing & Environment Requirements

- **Local Services**: `cargo test` requires PostgreSQL (port 5432) and Redis (port 6379) running on localhost.
  - Initialize using `./scripts/init_db.sh` and `./scripts/init_redis.sh` (or `SKIP_DOCKER=1 ./scripts/init_db.sh` if local instances exist).
- **Test Isolation**: Each integration test in `tests/api/` spawns an isolated PostgreSQL database with a UUID name, applies migrations automatically, and runs on an isolated random port with TLS disabled.
- **Test Modules**: Integration test modules must be registered in `tests/api/main.rs`.

## Configuration Gotchas

- **Local Config**: `configuration/local.yaml` sets `worker_enabled: false` to avoid background queue polling errors against placeholder Postmark tokens, and sets `base_url: "http://127.0.0.1:8080"`.
- **Environment Overrides**: Application settings can be overridden via environment variables using the `APP_` prefix and `__` separator (e.g. `APP_APPLICATION__PORT=8080`).
