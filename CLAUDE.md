# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Actix Web (Rust) backend serving a React/TypeScript SPA. Started from "Zero to Production in Rust"
by Luca Palmieri; now a personal site with newsletter subscriptions, an admin dashboard, contact
forms, and session-based auth.

## Architecture

**Backend** — entry points [src/main.rs](src/main.rs) (server) and
[src/issue_delivery_worker.rs](src/issue_delivery_worker.rs) (email queue worker, spawned when
`worker_enabled`). [src/startup.rs](src/startup.rs) owns TLS, middleware, and all route
registration. PostgreSQL via sqlx (compile-time checked queries), Redis-backed sessions, argon2
auth with middleware in [src/authentication/middleware.rs](src/authentication/middleware.rs),
Postmark email, and DB-backed request idempotency.

**Frontend** — [ui/](ui/), Vite + TypeScript, output to `ui/dist/`. MUI v6 on its default
**Emotion** engine; `styled-components` and `@mui/styled-engine-sc` are installed but unused, since
switching engines needs a `@mui/styled-engine` alias in `vite.config.ts` and there isn't one.
Routing is react-router v7 `createBrowserRouter` — real paths, no hash.

**Modules** — `routes/` (handlers by feature), `domain/` (validated newtypes), `idempotency/`,
`telemetry/`.

### UI builds are not automatic

[build.rs](build.rs) declares `cargo:rerun-if-changed` only for `ui/package.json`,
`ui/package-lock.json`, and `ui/build.rs` (which doesn't exist). Nothing covers `ui/src/**` or
`ui/index.html`, so editing UI source does not reliably re-run `npm run build`.

That matters because [src/routes/home/mod.rs](src/routes/home/mod.rs) embeds `ui/dist/index.html`
at **compile time** (`include_str!`) while `Files` serves content-hashed assets from **disk at
runtime**. When they drift, the HTML points at asset filenames that no longer exist → blank page.

**After changing anything under `ui/`, run `cd ui && npm run build` before `cargo run`.**

### SPA routing contract

Client routes are mirrored server-side so deep links and hard refreshes work. Three places move in
lockstep: `<Routes>` in [ui/src/App.tsx](ui/src/App.tsx), the `.route(path, ...to(home))` entries in
[src/startup.rs](src/startup.rs), and [public/sitemap.xml](public/sitemap.xml).

- Those routes must be registered **before** `Files::new("/", "./ui/dist/")` or the static handler wins.
- There is deliberately **no `default_handler`** — unknown paths fall through to `Files` and return a
  real 404. A blanket fallback would return 200 for everything and create soft 404s.

[tests/api/spa_routing.rs](tests/api/spa_routing.rs) locks both halves in.

## Development Commands

```bash
./scripts/init_db.sh          # Postgres in Docker + migrations (SKIP_DOCKER=1 to reuse an instance)
./scripts/init_redis.sh       # Redis in Docker

cargo run                     # serve on :8080
cargo test                    # needs DATABASE_URL exported
cargo test <name>             # single test; TEST_LOG=1 for output
cargo fmt
cargo clippy -- -D warnings   # exact form CI gates on (rust-clippy.yml:115)

SQLX_OFFLINE=true cargo check --tests           # build without a live DB (committed .sqlx/ cache)
cargo sqlx prepare --workspace -- --all-targets # refresh that cache after changing a query

cd ui && npm ci && npm run build   # npm run dev / preview also available
```

sqlx-cli: `cargo install --version='~0.8' sqlx-cli --no-default-features --features rustls,postgres`

### Gotchas

- **`./scripts/init_db.sh` fails in non-interactive shells** — `docker exec -it` at lines 56/60
  errors with *"cannot attach stdin to a TTY-enabled container"*. Fine in a real terminal;
  otherwise run those two `psql` steps without `-it`, then `sqlx database create && sqlx migrate run`.
- **CI lints without `--all-targets`**, so tests aren't linted. Running `cargo clippy --all-targets`
  locally surfaces many pre-existing `needless_borrow` warnings in `tests/api/` — not regressions.
- **`configuration/local.yaml` is tracked** despite the `*/local.yaml` rule in `.gitignore` (the rule
  doesn't apply to tracked files), so local tweaks ship to everyone if committed.

## Configuration

YAML in [configuration/](configuration/): `base.yaml` plus `local.yaml` / `production.yaml`, selected
by `APP_ENVIRONMENT` (default `local`). Env vars override with the `APP_` prefix and `__` separator
(e.g. `APP_APPLICATION__PORT`). Also honored: `DATABASE_URL`, `COOKIE_SECURE`.

TLS uses rustls with `security/cert.pem` and `security/key.pem`, gated by `tls_enabled` (false in
`base.yaml` and in tests). Docker can mount secrets via `-v '/hostpath/':'/run/secrets':'ro'`.

## Testing

Integration tests in [tests/api/](tests/api/), helpers in
[tests/api/helpers.rs](tests/api/helpers.rs). Each test spawns the real app on a random port with
TLS disabled, against a UUID-named database with migrations applied, plus a wiremock email server
and a randomly-credentialed test user.

## Database

Migrations in [migrations/](migrations/) — applied automatically in tests, `sqlx migrate run`
manually. Tables: `subscriptions`, `subscription_tokens`, `users`, `newsletter_issues`,
`issue_delivery_queue`, `idempotency`, `contacts`.

The delivery worker polls `issue_delivery_queue` every 10s when empty and uses
`FOR UPDATE SKIP LOCKED` so multiple workers can run; dequeue/send/delete happen in one transaction,
retrying after 1s on error.

## Known Improvements

From [README.md](README.md): expire idempotency keys; give `issue_delivery_queue` a retry count and
exponential backoff.
