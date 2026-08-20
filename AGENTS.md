# AGENTS.md

## Repository Overview & Architecture

- **Stack**: Actix Web (Rust) backend + React/TypeScript (Vite + MUI v6) SPA in `ui/`.
- **UI Build / Compile-Time Embed**: Rust embeds `ui/dist/index.html` and `ui/src/seo/routes.json` at compile time (`include_str!`), while serving hashed static assets from disk at runtime. `build.rs` only detects changes to `ui/package.json` / `ui/package-lock.json`, not `ui/src/**`. Always run `cd ui && npm run build` after editing UI code before compiling or running Rust.
- **SPA Route Lockstep**: Section routes (`/skills`, `/experience`, `/education`, `/portfolio`, `/open-source`, `/projects`) serve the scrolling one-pager shell with canonical URL `/`. Updating SPA routes requires changing 2 places in sync:
  1. Client routes in `ui/src/App.tsx`
  2. Server SPA route handlers in `src/startup.rs` (tested in `tests/api/spa_routing.rs`)
  - Note: `public/sitemap.xml` intentionally lists only `/` to avoid duplicate indexing.
- **SEO & SSR**: `ui/src/seo/routes.json` is the metadata source of truth. `src/routes/home/mod.rs` injects `<head>` SEO tags at `<!--SEO-->` and crawler `<h1>`/`<p>` markup at `<!--SSR-BODY-->`.

## Commands & Verification

### Frontend (`ui/`)
- `npm run lint` — ESLint 9 (includes `jsx-a11y`)
- `npm test` — Vitest unit tests
- `npm run build` — TypeScript check (`tsc`) and Vite bundle build

### Backend (Rust)
- `cargo test` — Run integration and unit tests (requires local Postgres & Redis)
- `cargo test <test_name>` — Run a single test (`TEST_LOG=1 cargo test <test_name>` for tracing logs)
- `cargo fmt --check` / `cargo fmt` — Code formatting
- `cargo clippy -- -D warnings` — Clippy lint (matches CI; running `--all-targets` flags pre-existing test warnings)
- `SQLX_OFFLINE=true cargo check --tests` — Compile and check tests without a running database
- `cargo sqlx prepare --workspace -- --all-targets` — Update `.sqlx/` query metadata cache after editing SQL queries

## Testing & Environment Requirements

- **Services**: `cargo test` requires Postgres (port 5432) and Redis (port 6379).
  - `./scripts/init_db.sh` (use `SKIP_DOCKER=1 ./scripts/init_db.sh` if Postgres is already running)
  - `./scripts/init_redis.sh`
- **Integration Test Isolation**: Tests in `tests/api/` automatically create a UUID-isolated Postgres database, run migrations, spin up an Actix server on a random port with TLS disabled, and mock Postmark email via WireMock.
- **Test Modules**: Every test file under `tests/api/` must be registered in `tests/api/main.rs`.

## Configuration Gotchas

- **Config Hierarchy**: Settings load from `configuration/base.yaml` + `configuration/{local,production}.yaml` based on `APP_ENVIRONMENT` (default `local`).
- **Environment Overrides**: Runtime overrides use the `APP_` prefix and `__` separator (e.g. `APP_APPLICATION__PORT=8080`).
- **Database Connection**: Application runtime parses the `database:` section in YAML configs; the `DATABASE_URL` env var is used only by `sqlx-cli` and compile-time query macros.
- **Local Worker**: `configuration/local.yaml` defaults `worker_enabled: false` to avoid queue polling errors against placeholder email tokens.
