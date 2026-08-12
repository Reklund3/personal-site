# Phase 6: SEO & Backend Route Reconciliation

## Objective
Reconcile the server-injected metadata, `sitemap.xml`, and the Rust integration tests with the single-page architecture, while keeping every deep-link URL at `HTTP 200`.

**This is the phase needing the most review care.** Collapsing 5 indexed URLs into one page is what
prevents five byte-identical pages from serving five different `<title>`s — the duplicate-content
problem that `/summary` caused and `2d01449`/`d56b1c5` just fixed.

---

## Detailed Tasks

### Task 6.1 — Collapse `ui/src/seo/routes.json` to one entry

```json
{
  "/": {
    "title": "Software Engineer",
    "h1": "Robert Eklund",
    "description": "Software engineer with expertise in Rust, functional programming, TypeScript, and distributed systems. Former finance professional turned full-stack developer.",
    "keywords": "Robert Eklund, Software Engineer, Rust, Functional Programming, TypeScript, React, Actix Web, Scala, DevOps, Backend Developer",
    "ogType": "profile",
    "includeProfileTags": true
  }
}
```

⚠️ **`title` is a prefix, not the whole tag.** Both renderers append the site name —
`home/mod.rs:39` builds `format!("{} | Robert Eklund", meta.title)` and `ui/src/utils/seo.tsx`
builds the same string client-side. A value like `"Robert Eklund — Software Engineer"` renders as
`Robert Eklund — Software Engineer | Robert Eklund`.

⚠️ Do not carry over today's `/education` description — it claims an MBA from St. Edward's
University and Austin Coding Academy, and neither appears on the site (see README content gaps).

**The other six paths need no entries.** `build_seo_tags` and `build_ssr_body` already do
`ROUTES.get(path).or_else(|| ROUTES.get("/"))` (`home/mod.rs:34-37`, `150-153`), so every unlisted
path falls back to `/` automatically. `SEOMetaTags` does the same client-side
(`routesMap[path] || routesMap['/']`). That fallback is what makes this a one-line data change
rather than a code change.

### Task 6.2 — One canonical, path-independent (`src/routes/home/mod.rs`)
The **only** required Rust edit is line 43:

```rust
// before
let canonical_url = format!("{}{}", site_url, path);
// after
let canonical_url = format!("{}/", site_url);
```

- `og:url` reuses `escaped_canonical` (line 84), so it collapses with it — no second edit.
- `build_ssr_body` needs **no change**: it already reads `meta.h1`, which Task 6.1 sets to
  `Robert Eklund`. It emits `<h1>…</h1>\n<p>…</p>`; the `<p>` stays.
- The `path` parameter of `build_seo_tags` stays in use for the metadata lookup — do not remove it.

### Task 6.3 — `public/sitemap.xml` → one `<loc>`
- Delete the `/skills`, `/experience`, `/education`, `/portfolio` blocks.
- Keep the apex `<url>` with its `<image:image>` headshot block, fix the stale "Homepage (Summary)"
  comment, and bump `lastmod`.
- ⚠️ The `<image:loc>` points at `https://www.roberteklund.us/headshot`, which **404s in production**
  (same gitignored-`data/` root cause as `/resume`). Pre-existing and independent of this work, but
  it also means the injected `og:image` / `twitter:image` currently unfurl with no image. Worth
  fixing alongside.

### Task 6.4 — `src/startup.rs` — keep all routes, update the comment
- Leave `web::resource("/").to(home)` (line 167) and the six `.route(…, web::get().to(home))`
  registrations (lines 195–201) exactly as they are: deep links must keep returning 200.
- Update the lockstep comment (lines 190–194) — it currently describes per-page documents. It should
  say all SPA paths serve one scrolling page with a single `/` canonical, and that the three places
  to keep in lockstep are `ui/src/App.tsx` `<Routes>`, this list, and `public/sitemap.xml`.
- **Keep the absence of a `default_handler`.** Unknown paths must keep falling through to `Files` and
  returning a real 404; a blanket fallback would return 200 for everything and create soft 404s.

### Task 6.5 — Update **both** tests in `tests/api/seo_metadata.rs`

**Test 1 — `server_renders_route_specific_seo_metadata` (lines 4–83).** Three edits, all required:

1. The five `(path, title, description)` cases (lines 7–33) collapse: every path now expects the same
   `<title>Software Engineer | Robert Eklund</title>` and the same description substring.
2. The canonical assertion (lines 66–69) currently interpolates the path:
   ```rust
   let expected_canonical = format!(r#"<link rel="canonical" href="{}{path}" />"#, test_app.base_url);
   ```
   It must drop `{path}` and become path-independent:
   ```rust
   let expected_canonical = format!(r#"<link rel="canonical" href="{}/" />"#, test_app.base_url);
   ```
   Leaving `{path}` in is the exact mistake that breaks the suite.
3. **`assert_ne!` at line 79 must be inverted.** It exists to prove injection is happening per route;
   under one canonical the equivalent proof is that every path returns a byte-identical title —
   `assert_eq!`. Rename the test accordingly (it is no longer "route-specific").

**Test 2 — `server_renders_an_h1_for_every_spa_route` (lines 91–136).** Earlier drafts of this plan
omitted this test entirely; it *will* fail as written. Its table (lines 95–101) maps each path to a
different heading (`<h1>Skills</h1>`, `<h1>Education</h1>`, …). Collapse it so every path expects
`<h1>Robert Eklund</h1>`. **Keep** the `assert_eq!(body.matches("<h1").count(), 1)` guard at
lines 129–134 — with one `<h1>` in the masthead it is more important, not less.

**`tests/api/spa_routing.rs`** — unchanged in intent (`SPA_PATHS` still 200, unknown paths still
404). Re-verify only. Consider adding `/open-source` and `/projects`, which no test currently covers.

---

## Files Changed
- `ui/src/seo/routes.json`
- `src/routes/home/mod.rs`
- `public/sitemap.xml`
- `src/startup.rs` *(comment only)*
- `tests/api/seo_metadata.rs`
- `tests/api/spa_routing.rs` *(re-verify)*

---

## Verification Steps
1. **`cd ui && npm run build` first.** `home/mod.rs` embeds `ui/dist/index.html` via `include_str!`
   at compile time, so `cargo test` otherwise asserts against a stale shell.
2. `cargo test` green (needs a running Postgres; the `query!` macros resolve their compile-time
   database from the tracked `.env`, or set `SQLX_OFFLINE=true` to use the committed `.sqlx/` cache).
3. `curl -s http://127.0.0.1:8080/skills | grep -i 'canonical\|og:url'` → both read
   `http://127.0.0.1:8080/`. (The port comes from `configuration/local.yaml`'s
   `base_url: "http://127.0.0.1:8080"`; with the bare `http://127.0.0.1` you would see no port.)
4. `curl -I http://127.0.0.1:8080/experience` → `200 OK`; `/open-source`, `/projects` → 200;
   `/nonsense` → 404.
5. Every SPA path emits exactly **one** canonical, one `<title>`, one JSON-LD block, and no leftover
   `<!--SEO-->` or `<!--SSR-BODY-->` marker.
6. The sitemap's URL list and the `startup.rs` route list agree with `App.tsx`'s `<Routes>`.
