# Plan: Editorial one-pager layout (design handoff 1c)

Source: `Alternative layout options.zip` → `design_handoff_editorial_layout/`, option **1c**
("Editorial one-pager", `Portfolio Layouts.dc.html` lines 389–512, content model at 807–916).
Replaces the 5-route tabbed SPA with one scrolling page: hero masthead → sticky scroll-spy nav →
5 stacked sections on alternating background bands → footer.

> **This is not a UI-only change.** Collapsing 5 indexed URLs into one page necessarily touches
> `src/routes/home/mod.rs`, `src/startup.rs`, `public/sitemap.xml`, `ui/src/seo/routes.json`,
> `tests/api/seo_metadata.rs`, and `tests/api/spa_routing.rs` — most of the SEO work landed in
> `2d01449`. A frontend-only implementation **will fail CI**.

---

## Decisions to make before implementation

### Decision 1 — what happens to `/skills`, `/experience`, `/education`, `/portfolio`? **(the crux)**

Each is currently a distinct indexed URL with its own server-injected `<title>`, description, and
self-canonical, and each is in `sitemap.xml`. If all five render one identical page, five URLs serve
byte-identical content — **the duplicate-content problem `/summary` caused**, just fixed this session.
The handoff (README line 68) asks only that deep links "scroll to the anchor"; it supplies no
canonical strategy.

| | **A — Deep links, one canonical** *(recommended)* | **B — Keep 5 indexed pages** | **C — Delete the routes** |
|---|---|---|---|
| `/skills` returns | 200, scrolls to anchor | 200, indexed separately | 404 |
| Canonical | all → `https://www.roberteklund.us/` | self | n/a |
| `sitemap.xml` | 1 URL | 5 URLs | 1 URL |
| `routes.json` | collapses to 1 entry | stays 7 | 1 entry |
| Duplicate content | none | **yes — 5 identical pages** | none |

**Recommend A.** Honors the deep-link requirement, keeps old links alive, one canonical. Real cost:
the per-route metadata table from `2d01449` mostly collapses to one entry (the *injection mechanism*
stays and still feeds `/`). **B is not viable as specified** — five different `<title>`s all
describing the same page is worse than the `/summary` problem, because the titles actively misdescribe.

### Decision 2 — does the fixed `ResponsiveAppBar` survive?

**The prototype contains no AppBar at all.** Its sticky nav is `top: 0` and its scroll offsets
(90px spy threshold, 8px scroll-to) assume nothing above it. The README nonetheless says to reuse
`ResponsiveAppBar`. These are incompatible as written. Options: (a) keep the AppBar and offset the
sticky nav by its height, (b) drop the AppBar and let the masthead + sticky nav be the whole header.
**Recommend (a)** — the AppBar carries the resume link, social links, and Contact CTA — but note it
makes the header tall (see the height constraint below).

### Decision 3 — two navs

`ResponsiveAppBar` already has section nav (`menuItemsTitles` = Skills/Experience/Education/Portfolio)
in desktop Row 2. The design adds a sticky strip with the same destinations. **Recommend:** AppBar
keeps identity + actions only (logo, social, resume, Contact); section nav moves entirely to the
scroll-spy strip. `menuItemsTitles` has exactly one consumer, so removing it is a single-file change.

### Decision 4 — dark-only?

`theme.tsx` sets `colorSchemes: { dark: true }`, so light mode is live and follows system preference.
The design specifies only dark values. **Recommend forcing dark** — anything else means inventing a
light palette the handoff doesn't provide.

Mechanism (verified against installed MUI 6.5.0): `colorSchemes: { dark: true }` alone yields
`colorSchemes: ['light','dark']`, `defaultColorScheme: 'light'`, `colorSchemeSelector: 'media'`.
Adding **`defaultColorScheme: 'dark'`** collapses it to a single dark scheme. `colorSchemes: { light:
false, dark: true }` **throws**; `palette: { mode: 'dark' }` also works.

### Decision 5 — the breakpoint (design does not specify one)

The prototype has **no media queries**; desktop/mobile is a manual preview toggle (`isMobile`), with
frame widths 1240 vs 390. **Recommend MUI `md` (900px)** as the switch for all responsive values —
but this is an inference, not a handoff decision.

### Decision 6 — two Skills chip colors drifted

Prototype vs repo: **JetBrains Suite** `#5b5b5b` vs `#000000`; **Miro** `#c9a800` vs `#FFD02F`.
The other 31 chips match. These two are exactly the problem cases (pure black is near-invisible on
`#1e1e1e`; white-on-`#FFD02F` fails contrast), so the drift reads as a deliberate accessibility fix.
README line 10 ("colors are final") contradicts line 43 ("see current `Skills.tsx`").
**Recommend taking the prototype's values.**

### Decision 7 — the "↓ Resume" pill **(blocked on a missing asset)**

⚠️ **`/resume` returns 404 in production right now.** Verified:
`curl -I https://www.roberteklund.us/resume` → **HTTP 404**. The cause is in the repo: `base.yaml`
points at `./data/Robert_eklund_resume.pdf`, but `data/` is **gitignored** (`.gitignore:31`) and the
**Dockerfile never copies it** into the runtime image (it copies only `site`, `ui/dist`, `public`,
`configuration`). `resume.rs` 404s with "Resume not available" when the file is missing.

So the AppBar's existing Resume button (desktop *and* mobile) is **already broken in production**,
and `Hero.tsx`'s "Resume download coming soon" tooltip was accurate — not stale. The prototype's
inert `<span>` is the correct reading of the current state.

**Options:** (a) ship the asset (mount `data/` or bake it into the image) and make the pill a live
`href="/resume"`; (b) keep the pill visually present but non-functional with the "coming soon"
tooltip. **Recommend (a) if the PDF exists** — a prominent hero CTA that 404s is worse than the
current tucked-away one. Do not make it a live link without fixing the asset first.

### Decision 8 — the footer

Design shows one line ending "Source (MIT)". Live `Copyright.tsx` states **CC BY-NC-SA 4.0**,
consistent with `public/ai.txt`. **Deliberate deviation: keep the existing license wording** —
"MIT" would misstate it. The prototype's footer is unlinked text and never references its own
`CONTENT.profile.github/linkedin`, so it is styling reference only. Sub-question: compress
`Copyright` to a single line (keeping CC BY-NC-SA), or keep the 4-column footer.

---

## Verified facts this plan rests on

**Theme** (MUI **6.5.0** installed; `package.json` declares `^6.3.1`)
- Dark `primary.main` = `blue[200]` = **`#90caf9`** — exactly the design accent. No override needed.
- Dark `background.default` **and** `.paper` are both `#121212` (`createPalette.js:62-64`).
- **`#1e1e1e` is already what ships.** It's not a token — it's the *rendered* result of MUI's dark
  elevation overlay on a default `<Paper elevation={1}>` (`getOverlayAlpha(1)=0.051` composited over
  `#121212` → `rgb(30,30,30)`). ⚠️ **Do not set `background.paper: '#1e1e1e'`** — the overlay stacks
  on top and you get ~`#292929`. Cards get the design value for free.
- **`#171717` is genuinely bespoke** — zero occurrences in MUI, and unreachable via any of the 25
  elevation steps. This is the one net-new surface token.
- **`#062341` must be an explicit override.** MUI computes `primary.contrastText` =
  `rgba(0,0,0,0.87)` for `#90caf9`. The override is safe (contrast 9.09:1), but `#062341` is only
  1.18:1 on `#121212` — usable **on the accent only**, never as body text.

**Integration constraints**
- **AppBar height is a snapshot that never updates.** `useLayoutEffect` + `offsetHeight`, no
  `ResizeObserver`, no resize listener; the value lives in `AppLayout` local state. Desktop is a
  two-row block (~110–120px), mobile a single Toolbar (~64px), so the stored value **goes stale when
  the breakpoint is crossed by resizing**. A sticky nav needing `top: appBarHeight` can't read it
  today (private to `AppLayout`) — needs a `ResizeObserver` + lifting via context.
- **AppBar `z-index` is 1100** (MUI default, no override). Give the sticky nav `zIndex.appBar - 1`
  so it scrolls *under* the header. Dialogs/menus portal to 1300 and are unaffected.
- **`position: sticky` is viable** — no ancestor sets `overflow` anywhere in `ui/src`.
- **`Container maxWidth="lg"` (`App.tsx:53`) + `padding: 1` (`main.tsx:28`) block full-bleed bands.**
- **`ContactDialog` API:** `{ dialogOpen: boolean; onClose: () => void }`. State is **private to
  `ResponsiveAppBar`** — the hero's "Contact Now" needs that state lifted (context) or a second instance.

**Dead code** — ⚠️ *this list has since gone stale.* As of `d56b1c5`, only **`Hero.tsx`** still
exists (zero import sites *and* absent from `ui/dist`, verified two ways). `components/Footer.tsx`
(never at `components/footer/Footer.tsx`), `pages/Projects.tsx`, and `pages/OpenSource.tsx` were all
deleted in `d56b1c5`; `MenuItemSelected.tsx` in `2d01449`. **Live:** `AppFooter.tsx` → `Copyright.tsx`.
`Hero.tsx` is **not** reusable despite the README calling it a revival — it's a horizontal split with
an `AccountCircle` icon and a disabled resume button. The masthead is a rewrite that borrows only copy.

**Content** — the prototype ships a complete `CONTENT` object (lines 807–916) with real copy, directly
liftable. Its `about.sections` use **`intro` → `bullets` → `outro`** ordering; a naive
`{heading, paragraphs[], bullets[]}` model would wrongly hoist Technical Journey's closing paragraph
above its bullets.

**Fidelity note:** where the README prose and the markup disagree, **the markup wins**. Notably the
eyebrow `margin-bottom` varies by section (About 14 / Skills 18 / Experience 22 / Education 14 /
Portfolio 18) though README line 38 implies a uniform 14px.

---

## Phase 1 — Theme foundation

1. Add **one** custom surface token for `#171717` (e.g. `palette.surface.alt`) with TypeScript
   **module augmentation** so it's typed. Do **not** add tokens for `#121212` (default) or `#1e1e1e`
   (free via Paper elevation).
2. Override **`primary.contrastText: '#062341'`** — the only palette override strictly required.
3. Resolve Decision 4 (force dark).
4. Add `components` defaults for the uniform radii (card 10, chip 12, pill 20).
5. Keep per-technology brand colors as **data**, not theme keys — they're content. Apply Decision 6.

## Phase 2 — Content extraction

6. Create `ui/src/content/` typed to the prototype's shape, **lifting from the prototype's `CONTENT`
   object** (lines 807–916) rather than re-scraping the five page components — then **diff against
   the repo** and reconcile the three known deltas (Decision 6 colors; "Master's" vs "Masters"
   apostrophe; capitalized "Coming soon.").
7. Model `about.sections` as `{heading, intro[], bullets[{label?, text}], outro[]}` — the
   intro/outro split is load-bearing. Include `about.intro` (hardcoded in the markup but present in
   the data model).
8. This phase makes the rest tractable: sections become data-driven renderers, and the SEO layer can
   draw description text from the same source instead of a parallel table.

## Phase 3 — Shell restructure

> **Task 10 is a prerequisite bug fix, not incidental redesign work.** The stale-height bug exists
> today; it's merely invisible because nothing depends on the value at a breakpoint boundary. The
> sticky nav can't be built correctly without it. Land it as its own commit (ideally its own PR) so
> it stays independently reviewable rather than buried in a redesign diff.

9. Remove `Container maxWidth="lg"` and `padding: 1` from the content wrapper so bands run
   edge-to-edge; add an inner `Container` **per section** for the 60/26px inset.
10. **(prerequisite)** Replace the one-shot AppBar measurement with a **`ResizeObserver`** and expose
    the height via context, so both the content offset and the sticky nav's `top` consume one live
    value.
11. Lift `contactDialogOpen` out of `ResponsiveAppBar` so the masthead CTA can open the same dialog.
12. Route table collapses per Decision 1: `/` renders the one-pager; other paths resolve to it plus
    an initial scroll to the matching anchor.

## Phase 4 — Sections

13. `Masthead` — centered, 88px `Avatar` with "RE" initials on `primary.main`, name/title/tagline,
    then two pills: contained "Contact Now" and outlined "↓ Resume" (`href="/resume"`, Decision 7).
    Radial-gradient background per spec.
14. `Section` wrapper — takes `id`, `eyebrow`, `band`, `eyebrowGap` (varies per section, see fidelity
    note); applies `scrollMarginTop = appBarHeight + navHeight` so anchor landings clear both bars.
    This replaces the prototype's manual offset math.
15. `About` — intro + 6 subsections; 4px dot bullets via `::before`.
16. `Skills` — `repeat(auto-fit, minmax(200px,1fr))` → `1fr` at `xs`; brand-colored chips
    (`fontWeight: 500`); outlined soft-skill pills below.
17. `Experience` — center rule `display: {xs:'none', md:'block'}`; jobs alternating at ~46% width.
    **Zigzag is presentational only** — DOM order stays chronological so reading order matches the
    mobile stack.
18. `Education` — centered, two lines.
19. `Portfolio` — `columns: {xs: 1, md: 2}`, `breakInside: 'avoid'`.

## Phase 5 — Sticky scroll-spy nav

20. `SectionNav` — sticky at `top: appBarHeight`, `zIndex: theme.zIndex.appBar - 1`, five buttons,
    active styled with the 2px `primary.main` bottom border.
21. **Re-base the scroll logic.** The prototype scrolls a fixed-height `div` and reads
    `e.target.scrollTop` / `el.offsetTop`; production scrolls the window. Use an
    **`IntersectionObserver`** rather than porting `scrollTop + 90`. This is a required adaptation,
    not an option. ⚠️ **Risk:** the observer is not behaviorally equivalent to the prototype's
    "last section whose top is ≤ scrollTop + 90" heuristic. A section shorter than the observer's
    effective band may never become active — **Education is two lines**, so this will bite. Tune
    `rootMargin` (e.g. a top-biased band) and verify Education highlights; fall back to a
    scroll-position calculation if it can't be made reliable.
22. Use **`useScrollTrigger`** (threshold 16, matching the prototype) for the scrolled
    background/shadow state.
23. Nav click sets active **immediately**, then smooth-scrolls; honor `prefers-reduced-motion`.
24. Update the URL to the section anchor on click so deep links and back/forward stay coherent.
    Use **router-aware navigation** (`useNavigate(..., { replace: true })`) — raw
    `history.replaceState` fights react-router's own history management and desyncs its location state.
25. Do **not** port `getSelectedOption()` — it's a fragile string round-trip, and its
    `color: 'primary'` (`ResponsiveAppBar.tsx:202`) is an **inert** `sx` value (not a palette leaf),
    so today's "active" state is conveyed only by underline/weight.

## Phase 6 — SEO / routing reconciliation *(assumes Decision 1 = A)*

26. `routes.json` → single `/` entry; `home/mod.rs` emits one canonical (`base_url + "/"`) for every
    SPA path.
27. `sitemap.xml` → one `<loc>`, keeping the `<image:image>` headshot block.
28. `startup.rs` → keep the six routes registered (deep links stay 200); update the lockstep comment,
    which no longer describes per-page documents.
29. `tests/api/seo_metadata.rs` → the `assert_ne!` on titles (line 79) **must be inverted**. Be
    precise or this fails CI: under A the test must assert every SPA path returns (a) a
    **byte-identical `<title>`**, and (b) the **same path-independent canonical**
    `format!(r#"<link rel="canonical" href="{}/" />"#, test_app.base_url)`. Note the canonical no
    longer interpolates `{path}` — keeping today's `format!(...{path}...)` is the exact mistake that
    will break the suite.
30. `tests/api/spa_routing.rs` → unchanged in intent (paths still 200); re-verify.
31. **Heading outline** — one `<h1>` (hero name); section eyebrows become `<h2>` despite their 11.5px
    styling; current per-page `<h1>`s demote to `<h3>`. This overlaps the heading cleanup you said
    you'd own — **coordinate, don't duplicate**. The `CardHeader` half is **still outstanding**:
    `d56b1c5` added page-level `<h1>`s but left `CardHeader` untouched, and its title slot defaults
    to `component: 'span'` (`CardHeader.js:113-114`), so card titles contribute nothing to the
    outline. Note `titleTypographyProps` is deprecated in v6 — the current form is
    `slotProps={{ title: { component: 'h3' } }}`.

## Phase 7 — Cleanup

32. Delete verified-dead `Hero.tsx`, `Footer.tsx`, `pages/Projects.tsx`, `pages/OpenSource.tsx`.
33. Delete `menuItemsTitles`/`constants.ts` if the AppBar sheds section nav (Decision 3), plus the
    dead `'Open Source'` and `'/'→'Summary'` branches in `handleMenuItemClick`/`getSelectedOption`
    that can never match a rendered label.
34. Remove `App.tsx`'s lazy-loading + prefetch machinery — with one page there's nothing to split
    (takes the `console.log('All components prefetched')` with it).
35. `handleLogoClick` currently no-ops when already on `/`. On a one-pager it must **scroll to top**,
    or clicking the logo will appear broken.

---

## Content gaps to resolve (pre-existing, surfaced by this work)

- **"Associate DevOps Engineer" has no bullets** — its content is literally "coming soon"
  (`Experience.tsx:81`). On tabbed pages it's easy to miss; on a single scrolling timeline it ships
  visibly. Fill or drop the entry before launch.
- **`routes.json` `/education` claims an "MBA from St. Edward's University" and Austin Coding
  Academy** — neither is on `Education.tsx` (two Texas State degrees only); Austin Coding Academy
  appears only in `Summary.tsx` prose, and the MBA appears nowhere. Confirm the MBA claim is accurate
  before carrying that text into the single description — metadata shouldn't assert credentials the
  site doesn't show.
- **`/headshot` also 404s in production** (`curl -I https://www.roberteklund.us/headshot` → **404**),
  same root cause as `/resume`: the `data/` directory is gitignored and never copied into the Docker
  image. Three consequences, all **pre-existing and independent of this redesign**:
  1. The server-injected **`og:image` and `twitter:image` point at a 404** — so the social-unfurl fix
     from `2d01449` currently produces previews with no image.
  2. `sitemap.xml`'s `<image:image>` block references the same dead URL.
  3. `ResponsiveAppBar`'s `Avatar src="/headshot"` is silently falling back to its `AccountCircle`
     `onError` handler.

  Worth fixing on its own merits, before or alongside this work. It also validates the design's
  choice of a text-initials ("RE") avatar over a photo — the README's "reuse any existing avatar
  image asset if the current site has one" resolves to: **there isn't a working one.**
- **Mobile has no GitHub/LinkedIn links today** (they live in a `md`-only Box). The redesign is a
  cheap place to fix that, but it's a behavior change, not a port.

## Verification

1. `cd ui && npm run build`; `cargo fmt --check`; `cargo clippy -- -D warnings`; `cargo test`.
2. Curl every SPA path: all 200, each emitting **one** canonical → `/`, one `<title>`, one JSON-LD
   block, no leftover `<!--SEO-->`.
3. Exactly one `<h1>` in the rendered DOM; gapless h1→h2→h3 outline.
4. Resize desktop↔mobile **without reloading** — sticky nav stays flush under the AppBar (this is
   the regression the current one-shot measurement would cause).
5. Keyboard: tab through nav; anchors land below both bars.
6. Reduced-motion: nav clicks jump instantly.
7. Mobile: bands full-bleed at 26px, timeline rule hidden, portfolio single column.
8. Confirm cards render `#1e1e1e` from the default Paper overlay — **not** ~`#292929`, which would
   mean `background.paper` was wrongly overridden.
9. Lighthouse a11y + SEO on `/` before and after.

## Sequencing

Separate branch and PR on top of merged `2d01449`. **Do not sweep in** the currently staged
`tests/api/hashed_asset_caching.rs` 304 test or the local-only `configuration/local.yaml` edit on
`seo-headings-and-asset-caching`. Land Phases 1–2 first (theme + content extraction are pure
refactors, independently reviewable); Phases 3–5 are the visible rewrite; Phase 6 is the SEO change
needing the most review care.
