# Phase 7: Cleanup & Verification

## Objective
Delete the code the one-pager makes dead, remove obsolete navigation constants, fix logo-click behavior, and run the full verification suite.

---

## Detailed Tasks

### Task 7.1 — Delete the five page components the sections replace
This is the bulk of the cleanup and was missing from earlier drafts of this plan. Phase 4 replaces
these with `sections/*`; leaving them behind means two copies of every string.

- `ui/src/components/pages/Summary.tsx`
- `ui/src/components/pages/Skills.tsx`
- `ui/src/components/pages/Experience.tsx`
- `ui/src/components/pages/Education.tsx`
- `ui/src/components/pages/Portfolio.tsx`
- `ui/src/components/PageTitle.tsx` — becomes dead with them (its only five consumers are the pages
  above). Its `variant="h4" component="h1"` pattern is worth carrying into the Masthead.

**Keep** `ui/src/components/pages/NotFound.tsx` (still the `*` route) and `ui/src/utils/seo.tsx`
(`SEOMetaTags` still sets `document.title`; with a one-entry `routes.json` its
`routesMap[path] || routesMap['/']` fallback keeps working unchanged).

### Task 7.2 — Delete verified-dead code
**Only one of the four files in earlier drafts still exists.** Verified against `HEAD` (`d56b1c5`):

| file | status |
|---|---|
| `ui/src/components/hero/Hero.tsx` | **still present — delete.** Zero import sites, absent from `ui/dist`. |
| `ui/src/components/Footer.tsx` | already deleted in `d56b1c5` (it was never at `components/footer/Footer.tsx`) |
| `ui/src/components/pages/Projects.tsx` | already deleted in `d56b1c5` |
| `ui/src/components/pages/OpenSource.tsx` | already deleted in `d56b1c5` |
| `ui/src/components/MenuItemSelected.tsx` | already deleted in `2d01449` |

`Hero.tsx` is **not** reusable as the masthead despite the handoff README calling it a revival — it
is a horizontal split with an `AccountCircle` icon and a disabled resume button. The Masthead is a
rewrite that borrows only copy.

**Live, keep:** `ui/src/components/footer/AppFooter.tsx` → `ui/src/components/Copyright.tsx`
(restyled in Phase 4).

### Task 7.3 — Remove obsolete navigation code
- Delete `ui/src/components/constants/constants.ts` — `menuItemsTitles` has exactly one consumer
  (`ResponsiveAppBar`), which Phase 3 stopped using.
- In `ResponsiveAppBar.tsx`, delete `handleMenuItemClick` (48–55) and `getSelectedOption` (70–75),
  including their branches that can never match a rendered label: `'Open Source'` (line 50) and
  `'/' → 'Summary'` (line 73).
- **`handleLogoClick` (61–68) must change behavior.** It currently no-ops when `pathname === "/"`.
  On a one-pager that makes the logo appear broken. Scroll to top instead, honoring reduced motion:
  ```ts
  window.scrollTo({ top: 0, behavior: prefersReducedMotion ? 'auto' : 'smooth' });
  ```
  Keep the `event.preventDefault()` and keep navigating when on some other path.

### Task 7.4 — Remove lazy loading and prefetching (`App.tsx`)
With one page there is nothing to code-split:
- Delete the six `lazy(() => import(...))` declarations (lines 8–16) and import the sections directly.
- Delete `prefetchComponents` (19–32), the `hasPrefetched` state and its `useEffect` (42–50) — this
  takes the stray `console.log('All components prefetched')` with it.
- Delete `LoadingComponent` (35–39) and the `<Suspense>` wrapper if nothing else needs it.

### Task 7.5 — Full verification suite
Run in this order — the frontend build **must** come first, because `src/routes/home/mod.rs` embeds
`ui/dist/index.html` with `include_str!` at compile time:

```bash
cd ui && npm run build && cd ..
cargo test                    # needs a running Postgres; see note below
cargo fmt --check
cargo clippy -- -D warnings   # CI's exact form — no --all-targets
```

`cargo clippy --all-targets` surfaces many pre-existing `needless_borrow` warnings in `tests/api/` —
those are not regressions from this work.

The tests provision their own per-test database (`helpers.rs:255` names it with a UUID; `299-325`
creates it and runs `sqlx::migrate!`), so no manual database setup is needed beyond a running
Postgres matching `init_db.sh`'s credentials. The separate requirement is at *compile* time: the
`sqlx::query!` macros verify SQL against a live database, resolved from the tracked `.env`. Set
`SQLX_OFFLINE=true` to compile against the committed `.sqlx/` cache instead.

### Task 7.6 — Material UI component audit
The prototype is raw `<div>`s with inline styles; the port must not be. Grep the new
`components/sections/` and `components/nav/` for constructs that have a Material UI equivalent:

- No `<div>` / `<span>` / `<ul>` / `<li>` / `<a>` where `Box`, `Typography`, `List`, `ListItem`, or
  `Link` applies. `<Box component="section">` is fine — that is the MUI way to pick the tag.
- No hand-rolled chip, card, avatar, tab, or button markup — all five ship in `@mui/material`.
- No `style={{…}}` attributes; styling goes through `sx` or theme overrides.
- Repeated type styles belong in the theme (the `eyebrow` variant), not copied into five `sx` props.
- No imports from `styled-components` or `@mui/styled-engine-sc` — installed but unused, and MUI
  runs on Emotion here.

### Task 7.7 — Accessibility & DOM audit
- Exactly one `<h1>` in the rendered DOM (`Robert Eklund`, in the Masthead) — both in the raw HTML
  the server sends and after React mounts.
- Gapless outline: `h1` (Masthead) → `h2` (section eyebrows, despite their 11.5px styling) → `h3`
  (About subsections, card titles).
- Keyboard: tab through `SectionNav`, anchors land clear of both bars, focus visible throughout.
- Lighthouse a11y + SEO on `/`, compared against a pre-change run.
- Check contrast on the two reconciled chip colors and confirm `#062341` is used **only** on the
  accent (it is 1.18:1 on `#121212`).

---

## Files Deleted
- `ui/src/components/pages/{Summary,Skills,Experience,Education,Portfolio}.tsx`
- `ui/src/components/PageTitle.tsx`
- `ui/src/components/hero/Hero.tsx`
- `ui/src/components/constants/constants.ts`

## Files Updated
- `ui/src/App.tsx`
- `ui/src/components/app-bar/ResponsiveAppBar.tsx`

---

## Final Check
Beyond a green suite, confirm the pre-existing issues this work surfaced are either fixed or
consciously deferred: the `/resume` and `/headshot` 404s, the "Associate DevOps Engineer" placeholder
bullet, the unverified MBA claim in the old `/education` metadata, and the missing mobile
GitHub/LinkedIn links.
