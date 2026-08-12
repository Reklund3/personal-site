# Editorial One-Pager Layout — Implementation Master Plan

Source Design: `design_handoff_editorial_layout/` (Option 1c — "Editorial one-pager")

This plan replaces the 5-route tabbed SPA with a unified, high-polish scrolling one-pager:
`Hero Masthead` → `Sticky Scroll-Spy Nav` → `5 Stacked Section Bands` (`About`, `Skills`, `Experience`, `Education`, `Portfolio`) → `Footer`.

> Full rationale, rejected alternatives, and the verification evidence behind every number below
> live in [`../../PLAN-editorial-layout.md`](../../PLAN-editorial-layout.md). These phase docs are
> the executable breakdown, not a replacement for it.

---

## Phase 0 — Prerequisites (do these first)

1. **Extract the design handoff.** Every line reference in these docs
   (`Portfolio Layouts.dc.html` lines 389–512 for the markup, 807–916 for `CONTENT`) points inside
   `Alternative layout options.zip`, which is **not extracted in the repo**. Unzip it somewhere
   outside the tree (it is untracked and should stay that way):
   ```bash
   unzip "Alternative layout options.zip" -d /tmp/handoff
   ```
2. **Branch from merged `main`.** As of 2026-08-11 that is `f578efe` ("Rename database to
   personal_site", #14) — one commit past the `d56b1c5` these docs were originally written against.
   The delta is the `newsletter` → `personal_site` database rename, which touches `.env`,
   `configuration/`, `spec.yaml`, `scripts/init_db.sh` and CI only; nothing in `ui/` or `src/routes/`,
   so every line reference in these phase docs still holds. Do not sweep in the untracked
   `traefik.yml`.
3. **Material UI is on v9.3.1 as of the branch's second commit.** The upgrade from 6.5.0 landed as
   its own commit before Phase 1, so every phase doc below is written against **9.3.1**. Two
   consequences that affect almost every file you will write:

   **(a) Deprecated system props were removed from `Box`, `Stack`, `Typography`, `Link`, `Grid`
   and `DialogContentText`.** Layout, spacing, display and flex shorthands must go in `sx`.
   Empirically verified against 9.3.1 — this is the exact boundary:

   | still legal | removed → use `sx` |
   |---|---|
   | `<Typography color="primary.main">` | `<Typography mb={2}>` |
   | `<Link color="primary" underline="none">` | `<Stack flexWrap="wrap">` |
   | `<Typography variant="h4" component="h1">` | `<Box display="flex">` |
   | `<Box component="section" id="x">` | `<Typography fontWeight={600}>` |
   | `<Stack direction spacing useFlexGap>` | any `mt`/`mb`/`px`/`py`/`justifyContent`/… |

   Note `color` **survives** — it is a real component prop, not just a system prop. It is only
   spacing/display/flex shorthands that moved. If you inherit v6-style code, the official codemod
   is `npx @mui/codemod@latest v9.0.0/system-props <path>`.

   **(b) `@mui/lab` is still not installed and still not an option** — see decision 10.

4. **Know the build order.** `src/routes/home/mod.rs` embeds `ui/dist/index.html` via `include_str!`
   at **compile time**, while `Files` serves hashed assets from disk at runtime. Any phase that
   touches `ui/` must run `cd ui && npm run build` **before** `cargo test` / `cargo run`, or the
   Rust side tests a stale shell. `cargo test` also needs a running Postgres — the tests provision
   their own per-test database (`helpers.rs:255`, `299-325`), but the `sqlx::query!` macros need a
   database at *compile* time, satisfied by the tracked `.env` or by `SQLX_OFFLINE=true`.

---

## Architectural Decisions Log

1. **Routing & SEO Canonical Strategy**:
   - Deep links (`/skills`, `/experience`, `/education`, `/portfolio`) return `HTTP 200`, load the one-pager, and automatically smooth-scroll to the target section anchor.
   - Legacy `/open-source` and `/projects` stay registered and land on the Portfolio anchor.
   - All SPA routes emit a single canonical URL (`https://www.roberteklund.us/`).
   - `sitemap.xml` collapses to 1 apex URL (`/`).
   - **There is no `/about` path.** The About section's anchor is the apex `/`. See Phase 5.
2. **App Bar & Navigation**:
   - `ResponsiveAppBar` remains fixed at the top for site identity (Logo), social links, Resume button, and "Contact Now" CTA button.
   - Desktop Row 2 section links are removed from `ResponsiveAppBar` and moved entirely into the sticky `SectionNav` strip.
   - Dynamic height measurement uses a `ResizeObserver` piped through React Context so `SectionNav` sticky offset (`top: appBarHeight`) updates live across window resizes and mobile/desktop breakpoint shifts.
3. **Theming** — force dark mode:
   - ⚠️ `colorSchemes: { dark: true }` is **what `theme.tsx` already has, and it does not force dark.**
     Verified against the installed MUI 9.3.1: that config yields `colorSchemes: ['light','dark']`,
     `defaultColorScheme: 'light'`, `colorSchemeSelector: 'media'` — light mode is live and follows
     system preference. Adding **`defaultColorScheme: 'dark'`** is what collapses it to a single
     dark scheme. See Phase 1.
   - Token strategy:
     - Default background: `#121212`
     - Net-new surface band token: `#171717` (`surface.alt`)
     - Card background: `#1e1e1e` (provided automatically by MUI Paper elevation 1 over `#121212`)
     - Primary contrast text: `#062341` (on `#90caf9` accent) — **must be an explicit override**;
       MUI computes `rgba(0,0,0,0.87)`.
4. **Content Model**:
   - Centralize structured site content into `ui/src/content/index.ts` with strict TypeScript types in `ui/src/content/types.ts`, shaped to the prototype's `CONTENT` object.
5. **SEO & Integration**:
   - Server-side injection (`src/routes/home/mod.rs`) emits unified metadata and SSR `<h1>` body for `/`.
   - Integration tests in `tests/api/seo_metadata.rs` assert all SPA routes render canonical `/` and matching meta tags. **Both** tests in that file need updating — see Phase 6.
6. **Breakpoint**: MUI `md` (900px) is the desktop/mobile switch for every responsive value. The
   prototype has no media queries — it uses a manual preview toggle at frame widths 1240 / 390 — so
   this is an inference, not a handoff decision.
7. **Skill chip colors**: take the prototype's `JetBrains Suite #5b5b5b` and `Miro #c9a800` over the
   repo's `#000000` / `#FFD02F`. The other 31 chips match. Pure black is near-invisible on `#1e1e1e`
   and white-on-`#FFD02F` fails contrast, so the drift reads as a deliberate accessibility fix.
8. **The "↓ Resume" pill is blocked on a missing asset.** `/resume` and `/headshot` both **404 in
   production**: `base.yaml` points at `./data/…`, `data/` is gitignored, and the Dockerfile never
   copies it into the runtime image. Ship the asset before wiring a prominent hero CTA to it. See
   Phase 4.
9. **Footer license wording**: the design's footer line ends "Source (MIT)". The live `Copyright.tsx`
   says **CC BY-NC-SA 4.0**, consistent with `public/ai.txt`. **Deliberate deviation — keep the
   existing license wording.** Restyle only. See Phase 4.
10. **Build from Material UI components, not hand-rolled markup.** The prototype is raw `<div>`s with
    inline styles; every one of its constructs has a Material UI equivalent, and the port should use
    it. Verified against the installed **9.3.1** via the MUI MCP docs and the package tree:

    | Design construct | Component | Availability |
    |---|---|---|
    | Sticky section nav w/ 2px active bar | `Tabs` / `Tab` (`role="navigation"`) | `@mui/material` ✅ |
    | Section eyebrow | custom `eyebrow` typography variant → `<h2>` | theme feature ✅ |
    | Skill chips, soft-skill pills | `Chip`, `Chip variant="outlined"` | `@mui/material` ✅ |
    | Button / chip rows that wrap | `Stack … useFlexGap sx={{flexWrap}}` | `@mui/material` ✅ |
    | About bullet lists | `List` / `ListItem` | `@mui/material` ✅ |
    | Project cards | `Card` / `CardContent` / `CardActions` / `Link` | `@mui/material` ✅ |
    | Nav scrolled background | `useScrollTrigger` | `@mui/material` ✅ |
    | Experience zigzag timeline | `Timeline position="alternate"` | `@mui/lab` ✅ **installed** |
    | Portfolio masonry | `Masonry sequential` | `@mui/lab` ✅ **installed** |

    **Decision: adopt `@mui/lab`, pinned at `9.0.0-beta.8`. Use `Timeline` for Experience and
    `Masonry` for Portfolio.**

    > *Decision history — this flipped twice, so here is what actually moved.* The first pass
    > recommended adopting. A registry check then found the v6-compatible release was beta-only and
    > pulled `@mui/base@5.0.0-beta.70`, which flipped it to decline. Upgrading to MUI 9.3.1 (Phase
    > 0.3) then invalidated that objection: measured, the v9-line lab adds **1 package with zero
    > transitive deps**, and `Timeline` in real use costs **+4.8 kB raw / +1.4 kB gzip**. What
    > remains true is only that lab is a permanently-prerelease channel — accepted deliberately.

    Facts worth keeping, all verified against the installed tree:

    - **`@mui/lab` has never shipped a stable release on any line** — `5.0.0-alpha.177`,
      `6.0.1-beta.36`, `7.0.1-beta.25`, `9.0.0-beta.8`. It is a permanent incubator, so "wait for
      stable" is not a strategy and never will be. **Pin the exact version** — a `^9.0.0-beta.8`
      caret range resolves forward across betas (`9.0.0-beta.20`, and any `9.x` stable), which is
      how a beta dependency breaks a build unattended. `package.json` carries `"9.0.0-beta.8"`.
    - **Upgrading Material UI does not make these available.** `Timeline` and `Masonry` are
      *documented* under the Material UI nav on mui.com — including for 9.3.1 — but they are not in
      the `@mui/material` package. Verified by `find` over clean installs of `9.3.1` and `7.3.11`:
      zero files matching `*timeline*` or `*masonry*`. The Timeline docs page agrees, linking
      `@mui/lab@latest` for bundle size. (The Masonry page links `@mui/material@latest` — that link
      is wrong.) `@mui/lab` is the only route, at any version.
    - `@mui/lab@9.0.0-beta.8` peers on `@mui/material: ^9.3.1`, matching what is installed exactly.

    The two overrides the design still needs are specified in Phase 4 Tasks 4.5 and 4.7 — the
    46% column split, the mobile collapse, and `sequential` ordering for Masonry. They are real
    work; adopting the components does not make them go away.

    Note `@mui/styled-engine-sc` and `styled-components` are installed but **unused** — MUI runs on
    its default Emotion engine, since switching would need a `@mui/styled-engine` alias in
    `vite.config.ts` and there isn't one. Do not import from them.

---

## Execution Phases & Tasks Breakdown

The implementation is broken down into 7 actionable phases:

1. [Phase 1 — Theme Foundation](./phase-1-theme-foundation.md): Dark theme, surface tokens, component overrides.
2. [Phase 2 — Content Extraction](./phase-2-content-extraction.md): Centralized data model & TypeScript types.
3. [Phase 3 — Shell Restructure](./phase-3-shell-restructure.md): Full-bleed layout, `ResizeObserver` height context, dialog state lifting, client route table.
4. [Phase 4 — Section Components](./phase-4-section-components.md): Masthead hero, 5 stacked section components, footer band.
5. [Phase 5 — Sticky Scroll-Spy Nav](./phase-5-sticky-scroll-spy-nav.md): Sticky `SectionNav`, `IntersectionObserver`, scroll-spy & URL anchor sync.
6. [Phase 6 — SEO & Backend Reconciliation](./phase-6-seo-backend-reconciliation.md): Backend routes, `routes.json`, `sitemap.xml`, Rust API tests.
7. [Phase 7 — Cleanup & Verification](./phase-7-cleanup-verification.md): Dead code removal, accessibility audit, full test suite validation.

**Suggested PR split:** Phases 1–2 are pure refactors and are independently reviewable. Phase 3's
`ResizeObserver` fix is a **pre-existing bug fix** and ideally lands as its own commit/PR. Phases 3–5
are the visible rewrite. Phase 6 is the SEO change needing the most review care.

---

## Content gaps to resolve before launch

These are pre-existing, surfaced by this work, and independent of the redesign:

- **"Associate DevOps Engineer" has no bullets** — its content is literally "coming soon"
  (`Experience.tsx:80`; the prototype carries it forward as `'Coming soon.'`). On tabbed pages this
  is easy to miss; on a single scrolling timeline it ships visibly. Fill or drop the entry.
- **`routes.json` `/education` claims an MBA from St. Edward's University and Austin Coding
  Academy** — neither appears on `Education.tsx` (two Texas State degrees only). Confirm before
  carrying any of it into the collapsed description.
- **`/resume` and `/headshot` 404 in production.** Beyond the hero CTA, this means the injected
  `og:image` / `twitter:image` and the sitemap `<image:image>` all point at a dead URL, and
  `ResponsiveAppBar`'s `Avatar src="/headshot"` is silently falling back to its `onError` handler.
- **Mobile has no GitHub/LinkedIn links** (they live in an `md`-only Box). Cheap to fix here, but
  it is a behavior change, not a port.

---

## Quick Verification Command Checklist

```bash
# Frontend build FIRST — home/mod.rs include_str!s ui/dist/index.html at compile time
cd ui && npm run build && cd ..

# Backend tests (needs a running Postgres; query! macros resolve via .env or SQLX_OFFLINE=true)
cargo test

# Formatting & Linter (CI's exact form — no --all-targets)
cargo fmt --check
cargo clippy -- -D warnings
```
