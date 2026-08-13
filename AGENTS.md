# AGENTS.md

## Repository Overview & Architecture

- **Stack**: Actix Web (Rust) backend + React/TypeScript (Vite + MUI) frontend in `ui/`.
- **Build Integration**: `cargo build` automatically executes `npm ci` and `npm run build` inside `ui/` via `build.rs`. Rust embeds `ui/dist/index.html` and `ui/src/seo/routes.json` at compile time using `include_str!`.
- **SPA Route Lockstep**: The application uses an Editorial One-Pager architecture where all section routes (`/skills`, `/experience`, `/education`, `/portfolio`) serve the scrolling one-pager shell with a single canonical URL (`/`). Adding or removing SPA section routes requires updating **2 places in lockstep**:
  1. Client routes in `ui/src/App.tsx`
  2. Server SPA routes in `src/startup.rs`
- **SEO Architecture**: Route metadata source of truth is `ui/src/seo/routes.json`. Backend (`src/routes/home/mod.rs`) injects `<head>` SEO tags (title, description, canonical `/`, OG, Twitter, JSON-LD) into `index.html` at `<!--SEO-->` and server-renders the `<h1>` body at `<!--SSR-BODY-->` for crawlers. Frontend (`ui/src/utils/seo.tsx`) updates `document.title` during client-side navigation. `public/sitemap.xml` contains a single apex entry for `/`.

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
