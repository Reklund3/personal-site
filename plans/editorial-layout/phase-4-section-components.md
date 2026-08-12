# Phase 4: Section Components

## Objective
Implement the Hero Masthead, the 5 stacked section components, and the footer band — applying alternating background bands, the design's inset, the typographic scale (`h1` → `h2` → `h3`), and responsive layouts.

All values below come from the prototype **markup** (`Portfolio Layouts.dc.html` lines 389–512).
Where the handoff README's prose and the markup disagree, **the markup wins**.

---

## Detailed Tasks

### Task 4.1 — Reusable `Section` wrapper (`ui/src/components/sections/Section.tsx`)
- Props: `id: string`, `eyebrow: string`, `band: 'default' | 'alt'`, `eyebrowGap: number`,
  `centerEyebrow?: boolean`.
- Outer `<Box component="section" id={id}>` — `bgcolor` is `background.default` (`#121212`) for
  `default` and `palette.surface.alt` (`#171717`) for `alt`. Use the Phase 1 token, not a literal.
- `scrollMarginTop: appBarHeight + navHeight` so anchor landings clear **both** bars. This replaces
  the prototype's manual offset math (its nav is `top: 0` with nothing above it) and is what lets
  Phase 5 use a plain `scrollIntoView`.
- Inner `<Container maxWidth="lg">` with the design inset — `px: { xs: '26px', md: '60px' }`,
  `py: '36px'` (Education overrides to `30px`).
- **Eyebrow:** `<Typography variant="eyebrow" color="primary.main" sx={{ mb: eyebrowGap }}>` — the custom
  variant registered in Phase 1 Task 1.5, whose `variantMapping` already renders it as `<h2>`. ⚠️ Do
  **not** use `variant="overline"` (12px / weight 400 / `0.08333em` — none of which match) and do not
  re-declare the type styles per section with `sx`.

  **`eyebrowGap` varies per section** — this is the fidelity note; the handoff README implies a
  uniform 14px but the markup does not:

  | section | gap | band | centered |
  |---|---|---|---|
  | About | 14px | `#121212` | no |
  | Skills | 18px | `#171717` | no |
  | Experience | 22px | `#121212` | **yes** |
  | Education | 14px | `#171717` | **yes** (whole section) |
  | Portfolio | 18px | `#121212` | no |

  Bands alternate `default / alt / default / alt / default`, then the footer band is `alt`.

### Task 4.2 — `Masthead.tsx`
- Band padding `48px 60px 34px` desktop / `48px 26px 34px` mobile, `textAlign: 'center'`.
- **Background: `radial-gradient(circle at 50% 0%, #1c2733, #121212 70%)`** — verified at markup
  line 398. (An earlier draft of this doc specified a `rgba(144,202,249,0.08) → transparent` accent
  wash; that value appears nowhere in the handoff.)
- Avatar: 88×88px, `borderRadius: '50%'`, `bgcolor: 'primary.main'`, initials **"RE"** in
  `primary.contrastText` (`#062341`), weight 700, 30px, `mb: '18px'`.
- Name: `<Typography variant="h4" component="h1">Robert Eklund</Typography>` with `fontWeight: 700`,
  `letterSpacing: '-0.5px'`, `color: '#fff'`, `mb: '6px'`. ⚠️ **`variant="h4"`, not `h3`** — MUI's
  `h4` is `2.125rem` = **34px**, exactly the design size; `h3` is 48px.
- Title: **"Software Engineer"** — 15px, `rgba(255,255,255,.6)`, `mb: '16px'`. The design carries no
  location line here.
- Tagline: 13.5px / line-height 1.7 / `rgba(255,255,255,.75)`, `maxWidth: 420px`, centered,
  `mb: '22px'` — "Building scalable systems with functional programming principles. Passionate about
  type safety, DevOps automation, and mentoring engineers."
- Button row: `<Stack direction="row" spacing="10px" useFlexGap sx={{ flexWrap: 'wrap', justifyContent: 'center' }}>`
  — `useFlexGap` is required for `spacing` to survive wrapping. ⚠️ `flexWrap` and `justifyContent`
  **must** be in `sx` on MUI 9 (see README Phase 0.3a); as bare props they no longer compile. Both pills are
  `borderRadius: 20px`, `padding: '10px 22px'`, `fontSize: 12.5px` — apply via `sx`, **not** a global
  `MuiButton` override (see Phase 1).
  - Contained "Contact Now" — `bgcolor: primary.main`, text `primary.contrastText`; calls
    `openDialog` from the Phase 3 context.
  - Outlined "↓ Resume" — `border: 1px solid rgba(255,255,255,.3)`, text `rgba(255,255,255,.75)`.

  ⚠️ **Do not wire `href="/resume"` yet.** `/resume` **404s in production**: `base.yaml` points at
  `./data/Robert_eklund_resume.pdf`, `data/` is gitignored, and the Dockerfile never copies it into
  the runtime image. The prototype's own element is an inert `<span>`, and `Hero.tsx`'s "Resume
  download coming soon" tooltip was accurate, not stale. Either ship the asset first (mount `data/`
  or bake it into the image) and then make it a live link, or render the pill non-interactive with
  the "coming soon" tooltip. A prominent hero CTA that 404s is worse than the tucked-away one that
  does today.

### Task 4.3 — `AboutSection.tsx`
- Eyebrow `ABOUT`, gap 14, band `default`.
- `content.about.intro` paragraph: 14px / 1.8 / `rgba(255,255,255,.8)`, `mb: '18px'`.
- 6 subsections (Background, Technical Journey, Key Achievements, DevOps & Infrastructure,
  Mentorship & Collaboration, Future Focus), each: `<Typography component="h3">` 13.5px bold
  `rgba(255,255,255,.88)` `margin: '16px 0 8px'`, then **`intro` → `bullets` → `outro` in that
  order** (the ordering is load-bearing — see Phase 2).
- Body paragraphs 13px / 1.7 / `rgba(255,255,255,.72)`.
- Bullets: render as `<List disablePadding>` / `<ListItem disableGutters>` — it is a list, and the
  MUI components give the right semantics without the default gutters fighting the design. Style
  each item 12.5px / 1.65, `pl: '14px'`, `position: relative`, with a `::before` 4px dot
  (`primary.main`, `position: absolute; left: 0; top: 7px; borderRadius: 50%`). Optional bold inline
  `label` renders as `<strong>{label}: </strong>` in `rgba(255,255,255,.88)`. Do not use
  `ListItemIcon` for the dot — it carries a min-width the design has no room for. (On MUI 9 that
  default dropped from 56px to `theme.spacing(4.5)` = **36px**, verified in
  `ListItemIcon/ListItemIcon.js:42`, so the objection is weaker than it was on v6 — but a 4px dot
  still does not need a 36px box. Keep the `::before`.)

### Task 4.4 — `SkillsSection.tsx`
- Eyebrow `SKILLS`, gap 18, band `alt`.
- Grid: `gridTemplateColumns: { xs: '1fr', md: 'repeat(auto-fit, minmax(200px, 1fr))' }`, `gap: 14px`.
- Category card: `<Card>` (elevation 1 → `#1e1e1e` for free), `borderRadius: 10` from the theme,
  `padding: 16px`; label 12.5px bold `rgba(255,255,255,.85)`, `mb: '10px'`.
- Chips: `<Chip size="small" label={item.name} />` inside a `<Stack direction="row" useFlexGap
  spacing="6px" sx={{ flexWrap: 'wrap' }}>`; per-chip `sx={{ bgcolor: item.color, color: '#fff' }}`, 11px.
  Radius 12 and `fontWeight: 500` come from the Phase 1 `MuiChip` override. Colors come from the
  content model (Phase 2).
- Soft-skill pills below, `mt: '16px'`, `gap: 8px`: `<Chip variant="outlined" size="small" />` —
  which already renders the design's `1px solid` border. Override only `borderRadius: 14px` (the
  theme default for chips is the brand-chip 12), `padding: '5px 12px'`, 11.5px,
  `color: rgba(255,255,255,.8)`.
- ⚠️ The current `Skills.tsx` already uses `<Chip>`; this section keeps that and drops only the
  per-category icons.
- The design drops the per-category MUI icons the current `Skills.tsx` renders.

### Task 4.5 — `ExperienceSection.tsx`

> **Resolved: use `@mui/lab`'s `Timeline`.** It is installed and pinned at `9.0.0-beta.8`.
> `position="alternate"` is exactly this zigzag and removes the index-parity math. See README
> decision 10 for why this flipped twice and what the measured cost was.

- Eyebrow `EXPERIENCE`, gap 22, **centered**, band `default`.
- Import from `@mui/lab`: `Timeline`, `TimelineItem`, `TimelineSeparator`, `TimelineConnector`,
  `TimelineContent`.
- **Omit `TimelineDot` entirely.** A `TimelineSeparator` holding only a `TimelineConnector` renders
  the design's uninterrupted centre rule; a dot would break it at every entry. Style the connector
  `width: '2px'`, `bgcolor: 'rgba(255,255,255,.15)'`.
- **The 46% split needs an override.** `TimelineItem` is `display: flex` and, when no
  `TimelineOppositeContent` is present, injects a `::before` pseudo with `flex: 1` as the spacer —
  so the default is 50/50 (verified, `TimelineItem/TimelineItem.js:49-53`). Override both sides on
  the `Timeline`'s `sx`:
  ```tsx
  <Timeline position="alternate" sx={{
    p: 0, m: 0,
    '& .MuiTimelineItem-root::before': { flex: '0 0 46%', p: 0 },
    '& .MuiTimelineContent-root': { flex: '0 0 46%', py: 0 },
  }}>
  ```
- **Mobile collapse is not built in.** `position` is a plain prop, not a responsive value, and
  `alternate` works by flipping `flex-direction: row-reverse` on `&:nth-of-type(even)`
  (`TimelineItem.js:57-66`). Below `md`, override in `sx` rather than swapping the prop from JS —
  a `useMediaQuery`-driven prop change re-renders on resize and can disagree with the SSR shell:
  ```tsx
  [theme.breakpoints.down('md')]: {
    '& .MuiTimelineItem-root': { flexDirection: 'row' },
    '& .MuiTimelineItem-root::before': { display: 'none' },
    '& .MuiTimelineContent-root': { flex: 1, textAlign: 'left' },
  }
  ```
  ⚠️ The `nth-of-type(even)` rule sets `textAlign: 'right'` on content; the mobile override must
  beat it, so keep it after and target the same specificity.
- Entry, inside `TimelineContent`: 14px bold title `rgba(255,255,255,.9)` as
  `<Typography component="h3">`; `{company} · {dates}` at 11.5px `rgba(255,255,255,.55)`
  `mb: '8px'`; bullets 12px / 1.6 / `rgba(255,255,255,.72)` `mb: '4px'`.
- **Zigzag stays presentational.** `Timeline` renders `<ul>`/`<li>`, so DOM order is already
  chronological and matches the mobile stack — never reorder the array to achieve the visual.
- ⚠️ The 4th entry (Associate DevOps Engineer, 03/2019–03/2020) has a single `'Coming soon.'` bullet.
  It ships visibly on a continuous timeline — resolve the content before launch.
- ⚠️ `TimelineItem` sets `minHeight: 70` (`TimelineItem.js:48`). If an entry renders shorter than
  that the spacing will not match the design's `mb: '26px'` rhythm — override if it bites.

### Task 4.6 — `EducationSection.tsx`
- Eyebrow `EDUCATION`, gap 14, band `alt`, `textAlign: 'center'`, and section `py: '30px'` (not 36).
- **Two lines**, 13px / 1.7 / `rgba(255,255,255,.75)`, `mb: '6px'` each:
  1. Texas State University, San Marcos, Tx — Master's in Accounting Information Systems
  2. Texas State University, San Marcos, Tx — Bachelor's in Accounting

  Note the apostrophe in "Master's" (the repo currently reads "Masters") and that the **Bachelor's
  line is not optional** — the design renders both.

### Task 4.7 — `PortfolioSection.tsx`
- Eyebrow `PORTFOLIO`, gap 18, band `default`.
- **Use `@mui/lab`'s `<Masonry>`** — adopted alongside `Timeline` (README decision 10):
  ```tsx
  <Masonry columns={{ xs: 1, md: 2 }} spacing={1.75} sequential>
  ```
  `spacing={1.75}` is 14px on the default 8px spacing unit, matching the design's `columnGap`.
  `Masonry` supplies the gaps, so drop the per-card `mb: '14px'` and `breakInside: 'avoid'` that a
  CSS-`columns` implementation would need.
- ⚠️ **`sequential` is required, not optional.** By default `Masonry` measures in JS and drops each
  item into the *shortest* column, which reorders the cards relative to the content array.
  `sequential` fills left-to-right in array order instead (`Masonry.js:161, 226`). Without it the
  open-source entry can float above the personal projects, contradicting the section's intended
  reading order.
- ⚠️ `Masonry` measures on the client, so the first paint before measurement can differ from the
  settled layout. The props that control the pre-measurement guess are `defaultColumns`,
  `defaultSpacing` and `defaultHeight`; set `defaultColumns={2} defaultSpacing={1.75}` so the
  server-rendered shell and the hydrated layout agree.
- Card: `<Card>` (elevation 1) wrapping `<CardContent>`, with the link in `<CardActions>` — use the
  MUI subcomponents rather than padding a bare `Card`, then zero out `CardContent`'s default
  `padding-bottom: 24px` and `CardActions`' 8px gutters to hit the design's flat 16px.
  `borderRadius: 10` comes from the Phase 1 override. Title 13.5px bold `rgba(255,255,255,.9)`;
  optional `subheader` 11px `rgba(255,255,255,.5)` `mb: '8px'`; paragraphs 12px / 1.6 /
  `rgba(255,255,255,.7)`; link via `<Link color="primary" underline="none" target="_blank"
  rel="noopener">` labelled `linkLabel ?? 'View on GitHub'` followed by `→`.
  ⚠️ Do **not** use `<CardHeader>` for the title. Verified in `@mui/material` **9.3.1**
  (`CardHeader/CardHeader.js:112-113`), its title slot still defaults to `variant: 'h5'`,
  **`component: 'span'`** — visual size only, no heading element. That is why today's card titles in
  `Experience.tsx` and `Portfolio.tsx` contribute nothing to the heading outline, and `d56b1c5` did
  **not** address it (it added page-level `<h1>`s and left `CardHeader` untouched). Render the title
  as `<Typography variant="h3">` explicitly instead.

  If you do keep a `CardHeader` anywhere: `titleTypographyProps` was deprecated in v6 and is now
  **removed outright in v9** — verified, zero occurrences in `CardHeader.js`. The only form that
  works is `slotProps={{ title: { component: 'h3' } }}`. (Codemod:
  `npx @mui/codemod@latest deprecations/card-header-props <path>`.)
- 3 personal projects + 1 open-source entry (Akka ActorTestkit — the one with both a `subheader` and
  a custom `linkLabel`).

### Task 4.8 — Footer band
The design's 8th band, missing from earlier drafts of this plan.
- Band `alt` (`#171717`), `padding: '22px 60px'` / `'22px 26px'`, `textAlign: 'center'`,
  `borderTop: 1px solid rgba(255,255,255,.1)`, single line at 10.5px `rgba(255,255,255,.5)`.
- **Restyle the existing `AppFooter.tsx` → `Copyright.tsx`; do not rebuild.** Sub-question left
  open: compress the current 4-column footer to the design's single line, or keep the columns.
- ⚠️ **Keep the existing CC BY-NC-SA 4.0 wording.** The design's line ends "Source (MIT)", which
  would misstate the license (`public/ai.txt` agrees with `Copyright.tsx`). Deliberate deviation.

---

## Files Created/Changed
- `ui/src/components/sections/Section.tsx` *(new)*
- `ui/src/components/sections/Masthead.tsx` *(new)*
- `ui/src/components/sections/AboutSection.tsx` *(new)*
- `ui/src/components/sections/SkillsSection.tsx` *(new)*
- `ui/src/components/sections/ExperienceSection.tsx` *(new)*
- `ui/src/components/sections/EducationSection.tsx` *(new)*
- `ui/src/components/sections/PortfolioSection.tsx` *(new)*
- `ui/src/components/footer/AppFooter.tsx`, `ui/src/components/Copyright.tsx` *(restyle)*

---

## Verification Steps
1. `cd ui && npm run build`.
2. Heading outline: exactly one `<h1>` (Masthead name), section eyebrows are `<h2>`, About
   subsection and card titles are `<h3>` — gapless, no skipped level.
3. Bands alternate `#121212 / #171717 / #121212 / #171717 / #121212` with the footer on `#171717`.
4. Cards compute to `rgb(30, 30, 30)` from the default Paper overlay — not `#292929`.
5. Responsive sweep at 390 / 768 / 1200px: timeline collapses to a single left-aligned column below
   `md` (no zigzag, no `::before` spacer, content left-aligned — the `nth-of-type(even)` override),
   portfolio single column, skills grid single column, inset drops 60px → 26px.
7. Portfolio card order matches the content array top-to-bottom (`sequential`), and does not
   visibly re-shuffle between first paint and settled layout.
6. Deep-link an anchor and confirm the heading clears **both** the AppBar and the sticky nav
   (`scrollMarginTop` correct).
