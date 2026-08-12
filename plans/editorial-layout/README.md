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
   | `<Typography variant="h4" component="h1">` | `<Typography mb={2}>` |
   | `<Box component="section" id="x">` | `<Stack flexWrap="wrap">` |
   | `<Stack direction spacing useFlexGap>` | `<Box display="flex">` |
   | `<Typography color="primary">` *(bare name only — see below)* | `<Typography fontWeight={600}>` |
   |  | any `mt`/`mb`/`px`/`py`/`justifyContent`/… |

   If you inherit v6-style code, the official codemod is
   `npx @mui/codemod@latest v9.0.0/system-props <path>`.

   ⚠️ **`color` on `Typography` is a trap — it survives as a prop but no longer accepts dotted
   paths.** In v9 it is matched against a generated variants list built from the palette keys
   (`Typography/Typography.js:62-74`), so it accepts **only bare names**. A dotted path matches
   nothing and is **silently discarded** — no type error, no runtime warning, no failing test.
   Verified against the installed 9.3.1, the complete accepted set is:

   ```
   primary  secondary  error  warning  info  success
   textPrimary  textSecondary  textDisabled  textIcon
   ```

   ```tsx
   <Typography color="primary.main">      // ❌ silently renders with NO color
   <Typography color="text.secondary">    // ❌ silently renders with NO color
   <Typography color="primary">           // ✅
   <Typography color="textSecondary">     // ✅
   <Typography sx={{ color: 'primary.main' }}>   // ✅ dotted paths work in sx
   ```

   **When in doubt use `sx`** — it resolves dotted palette paths correctly and has no such
   restriction. Grep for this before declaring a phase done, and note the JSX is often multi-line
   so a single-line pattern will miss it:
   ```bash
   grep -rnA3 '<Typography' ui/src/ | grep -E 'color="[a-zA-Z]+\.[a-zA-Z]+"'
   ```

   **(b) `@mui/lab` is installed**, pinned at `9.0.0-beta.8`, supplying `Timeline` and `Masonry` —
   see decision 10.

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
2. **App Bar & Navigation** — ⚠️ **REVISED during implementation. `ResponsiveAppBar` was deleted
   entirely; the site has no fixed header.**

   *The original decision was to keep the AppBar for identity, social links, Resume and "Contact
   Now". In practice that duplicated the one-pager's own content: the AppBar avatar restated the
   Masthead's hero portrait, and its menu items restated the section nav. On a single scrolling
   page both are redundant. The AppBar was removed and its responsibilities redistributed.*

   Where each responsibility now lives:

   | was in `ResponsiveAppBar` | now |
   |---|---|
   | Hero portrait / avatar | Masthead avatar (the only one) |
   | Section menu items | `SectionNav` (the only one) |
   | "Contact Now" CTA | Masthead pill → `ContactDialogContext` |
   | Resume button | Masthead "↓ Resume" pill |
   | GitHub / LinkedIn links | `Copyright.tsx` in the footer |

   Consequences that follow from this and are already reflected in the code:
   - **`SectionNav` sticks at `top: 0`**, not `top: appBarHeight` — there is nothing above it.
   - **`Section`'s `scrollMarginTop` is `navHeight` alone.** Anchors clear one bar, not two.
   - **`AppBarHeightContext` and the whole `ResizeObserver` height-measurement chain are gone.**
     This voids Phase 3 Tasks 3.1, 3.2 and 3.6, and Phase 7 Task 7.3's `handleLogoClick` change —
     all of them target deleted code. Phase 3's framing of the stale-height `ResizeObserver` fix as
     "a prerequisite bug fix" no longer applies: the buggy component does not exist.
   - **Moving the social links to the footer resolves the "mobile has no GitHub/LinkedIn links"
     content gap** listed below — they now render at every breakpoint.

   The phase documents have **not** yet been rewritten for this; they still describe the AppBar
   architecture in ~46 places. Treat this decision as authoritative where they disagree.
3. **Theming** — force dark mode:
   - ⚠️ **Phase 1 Tasks 1.1 and 1.2 contradict each other; 1.2 wins.** Task 1.1 says "do not set
     `primary.main`" while Task 1.2 requires overriding `primary.contrastText`. That combination is
     impossible: `SimplePaletteColorOptions` declares **`main: string` as required**, so a partial
     `primary: { contrastText }` does not typecheck. `main` must be supplied. `theme.tsx` sets it
     from `blue[200]` — MUI's own dark default — so the value stays visibly tied to the framework
     default rather than forking the accent into a second literal.
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
8. **The "↓ Resume" pill links straight to `/resume`.** ⚠️ **RETRACTED — an earlier version of this
   decision claimed `/resume` and `/headshot` 404 in production and told Phase 4 not to wire the
   link. That was wrong**, and Phase 4 Task 4.2 still carries the bad warning.

   The claim was inferred purely from repo contents — `base.yaml` pointing at `./data/…`, `data/`
   being gitignored, and the Dockerfile's runtime stage not copying it. **A repo snapshot cannot
   settle how an asset is provisioned at deploy time.** Two pieces of evidence in the repo
   contradicted the inference and were not checked:

   - `/resume` and `/headshot` are **real registered routes** — `src/startup.rs:185-186` wire them
     to `serve_resume` / `serve_headshot`, which read `resume_file_path` / `headshot_file_path`.
   - The previous live `ResponsiveAppBar` shipped a plain `href="/resume"` link in **both** desktop
     and mobile for its entire life, with no guard and no tooltip.

   Both work in production. The `Hero.tsx` "Resume download coming soon" tooltip that the original
   reasoning leaned on came from a **dead, never-imported component** — it was stale, not accurate.
   Wire the pill normally; no asset work is a prerequisite.
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
- ~~**`/resume` and `/headshot` 404 in production.**~~ **NOT A GAP — retracted.** Both are served
  by real routes (`src/startup.rs:185-186`) and work in production. See decision 8 for why this was
  wrongly reported, and what evidence was missed.
- ~~**Mobile has no GitHub/LinkedIn links.**~~ **RESOLVED** by decision 2 — the links moved from
  the deleted AppBar's `md`-only Box into `Copyright.tsx`, so they now render at every breakpoint.

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
