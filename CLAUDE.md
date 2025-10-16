# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust web application built with Actix Web serving a React/TypeScript frontend. Originally started as a learning project following "Zero to Production in Rust" by Luca Palmieri, this personal site includes newsletter subscription functionality, admin dashboard, contact forms, and session-based authentication.

## Architecture

### Backend (Rust/Actix Web)
- **Entry Points**:
  - [src/main.rs](src/main.rs) - Main web server
  - [src/issue_delivery_worker.rs](src/issue_delivery_worker.rs) - Background worker for email delivery
- **Application Startup**: [src/startup.rs](src/startup.rs) handles server initialization, TLS configuration, middleware setup, and route registration
- **Configuration**: Multi-environment YAML-based config in [configuration/](configuration/) directory
  - Supports environment variables with `APP_` prefix and `__` separator (e.g., `APP_APPLICATION__PORT`)
  - Environments: `local` and `production` (controlled by `APP_ENVIRONMENT`)
- **Database**: PostgreSQL with sqlx for compile-time checked queries
- **Session Management**: Redis-backed sessions via actix-session
- **Authentication**: Session-based auth with argon2 password hashing; middleware in [src/authentication/middleware.rs](src/authentication/middleware.rs)
- **Email**: Postmark integration for transactional emails and newsletters
- **Idempotency**: Request idempotency tracking in database to prevent duplicate operations

### Frontend (React/TypeScript)
- **Location**: [ui/](ui/) directory
- **Build**: Vite + TypeScript, built automatically during `cargo build` via [build.rs](build.rs)
- **Output**: [ui/dist/](ui/dist/) served as static files by Actix
- **UI Framework**: Material-UI (MUI) with styled-components

### Key Modules
- **routes/**: HTTP handlers organized by feature (admin, login, subscriptions, contact, health check)
- **domain/**: Domain models with validation (UserEmail, UserName, NewSubscriber, ContactMessage)
- **idempotency/**: Idempotency key handling and persistence
- **telemetry/**: Tracing/logging setup using tracing-subscriber

## Development Commands

### Database Setup
```bash
# Initialize PostgreSQL database (uses Docker by default)
./scripts/init_db.sh

# Initialize Redis (uses Docker by default)
./scripts/init_redis.sh

# Skip Docker if you have existing instances
SKIP_DOCKER=1 ./scripts/init_db.sh
```

### Rust Backend
```bash
# Build the project (includes UI build via build.rs)
cargo build

# Run the server (requires DATABASE_URL and configuration)
cargo run

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Run tests with output
TEST_LOG=1 cargo test

# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Database migrations (requires sqlx-cli)
# Install: cargo install --version='~0.8' sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run
```

### Frontend
```bash
cd ui

# Install dependencies
npm ci

# Development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## TLS/HTTPS Configuration

The application supports TLS with rustls. Certificate and key files are required:
- Default paths: `security/cert.pem` and `security/key.pem`
- Configurable via `ApplicationSettings.cert_file_path` and `key_file_path`
- TLS is disabled in tests via the `tls_enabled` flag in `Application::build()`
- Docker deployments can mount secrets from host via `-v '/hostpath/':'/run/secrets':'ro'`

## Environment Configuration

Required environment variables for local development (or set in configuration YAML):
- `DATABASE_URL`: PostgreSQL connection string
- `APP_ENVIRONMENT`: `local` or `production` (defaults to `local`)
- `COOKIE_SECURE`: Set to `"true"` for HTTPS-only cookies

Configuration files in [configuration/](configuration/):
- [base.yaml](configuration/base.yaml): Shared configuration
- `local.yaml` or `production.yaml`: Environment-specific overrides

## Testing

- Integration tests in [tests/api/](tests/api/)
- Test helper utilities in [tests/api/helpers.rs](tests/api/helpers.rs)
- Each test creates isolated database with UUID name
- Test app spawned with TLS disabled and random port
- Mock email server using wiremock
- Test user automatically created with random credentials

## Database Migrations

Located in [migrations/](migrations/) directory. Applied automatically during tests. Run manually with `sqlx migrate run`.

Key tables:
- `subscriptions`: Newsletter subscribers with confirmation status
- `subscription_tokens`: Confirmation tokens
- `users`: Admin users with hashed passwords
- `newsletter_issues`: Newsletter content
- `issue_delivery_queue`: Email delivery queue
- `idempotency`: Request idempotency tracking
- `contacts`: Contact form submissions

## Background Worker

The issue delivery worker ([src/issue_delivery_worker.rs](src/issue_delivery_worker.rs)) processes the email delivery queue:
- Polls `issue_delivery_queue` table every 10 seconds when empty
- Uses PostgreSQL `FOR UPDATE SKIP LOCKED` for concurrent worker support
- Dequeues, sends email, and deletes task in transaction
- Includes retry logic with 1-second delay on error

## Known Improvements

From [README.md](README.md):
- Need to expire idempotency keys
- Enhance issue_delivery_queue to use retry count and exponential backoff
