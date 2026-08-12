# Phase 5: Sticky Scroll-Spy Nav (`SectionNav`)

## Objective
Build the sticky navigation strip (`SectionNav`) that anchors below `ResponsiveAppBar`, tracks the active section with an `IntersectionObserver`, smooth-scrolls on click, and keeps the URL coherent with the visible section.

> **The scroll logic must be re-based, not ported.** The prototype scrolls a fixed-height `div` and
> reads `e.target.scrollTop` / `el.offsetTop`; production scrolls the window. This is a required
> adaptation, not an option.

---

## Detailed Tasks

### Task 5.1 — `SectionNav.tsx` layout (`ui/src/components/nav/SectionNav.tsx`)
- `position: 'sticky'`, `top: appBarHeight` (from the Phase 3 context — **not** `0`; the prototype
  has no AppBar above it), `zIndex: theme.zIndex.appBar - 1` so it scrolls *under* the fixed header.
  `zIndex.appBar` is MUI's default **1100** (no override in `theme.tsx`); dialogs and menus portal to
  1300 and are unaffected.
- `position: sticky` is viable — no ancestor sets `overflow` anywhere in `ui/src`.
**Use MUI `<Tabs>` / `<Tab>`, not hand-rolled buttons.** The design's active treatment — a 2px
`primary.main` bar under the current item — is exactly what MUI's built-in `TabIndicator` renders,
and `Tabs` also brings arrow-key navigation and focus management for free.

```tsx
<Tabs
  value={activeSection}                 // 'about' | 'skills' | …
  role="navigation"                     // NOT a tablist — see below
  aria-label="Section navigation"
  centered
  textColor="primary"
  indicatorColor="primary"
>
  {CONTENT.navItems.map(({ id, label }) => (
    <Tab key={id} value={id} label={label} component={RouterLink} to={SECTION_PATHS[id]} />
  ))}
</Tabs>
```

- **`role="navigation"` is deliberate.** MUI documents this "Nav tabs" pattern for tabbed
  *navigation*: the default `role="tablist"` promises tab panels that show and hide, which is not
  what happens here — the page scrolls. Do **not** wire up `aria-controls` / `role="tabpanel"`.
- `component={RouterLink}` keeps clicks in react-router rather than triggering a full page load
  (MUI's routing-integration guide covers this for `Tab` specifically).
- ⚠️ **`value` must match a `Tab`'s `value` or MUI logs a console error.** If the scroll-spy has no
  match yet, pass `value={false}` — the documented "nothing selected" sentinel — rather than `null`
  or `''`.
**MUI 9 notes for this component** (the phase is built entirely on `Tabs`, and v9 touched it):
- `TabIndicatorProps` is **removed** — if you style the indicator via props rather than a theme
  override or `sx`, the form is now `slotProps={{ indicator: … }}`. Same for `ScrollButtonComponent`
  → `slots.scrollButtons` and `TabScrollButtonProps` → `slotProps.scrollButtons`.
  (Codemod: `npx @mui/codemod@latest deprecations/tabs-props <path>`.)
- A `Tab` rendered **outside** a `Tabs` now throws rather than warning. Keep them nested.
- Keyboard nav now moves `tabindex` along with DOM focus (one focusable `Tab` at a time). This is
  an a11y improvement and needs no code, but it does change what a focus-order test would observe.

- Styling to reach the design: `.MuiTabs-indicator` to `height: 2px`; `MuiTab` label to 12px,
  `textTransform: 'none'`, weight 600 inactive / 700 active, `padding: '8px 12px 6px'`; inactive
  color `rgba(255,255,255,.65)`.
- The design wraps nav items on mobile. `Tabs` does not wrap — use
  `variant="scrollable" scrollButtons={false}` below `md` instead, which is the MUI-native answer to
  the same overflow problem.
- `borderBottom: 1px solid rgba(255,255,255,.08)`.
- Scrolled state: background `rgba(18,18,18,.8)` at rest → `rgba(15,15,15,.97)` plus
  `boxShadow: 0 6px 20px rgba(0,0,0,.45)` once scrolled; both transition `.25s ease`.
- Measure the nav's own height and publish it too — `Section`'s `scrollMarginTop` is
  `appBarHeight + navHeight` (Phase 4). Reuse the Phase 3 context pattern rather than hardcoding.

### Task 5.2 — Scrolled state via `useScrollTrigger`
```ts
const scrolled = useScrollTrigger({ threshold: 16, disableHysteresis: true });
```
⚠️ **`disableHysteresis: true` is required.** By default `useScrollTrigger` is direction-sensitive
(it compares against the previous scroll position), so without it the nav background flickers back to
its at-rest state when scrolling up mid-page. The design wants a plain "scrolled past ~16px" test.

### Task 5.3 — `IntersectionObserver` scroll-spy
- Observe the 5 section elements (`#about`, `#skills`, `#experience`, `#education`, `#portfolio`).
- Use a top-biased band, e.g. `rootMargin: '-20% 0px -60% 0px'`, so "active" means "near the top of
  the viewport" rather than "anywhere on screen".
- **Tie-break explicitly:** when several entries are intersecting, pick the one with the smallest
  positive `boundingClientRect.top` (the topmost). Without a rule, the active item depends on
  observer callback ordering.
- ⚠️ **Risk — `Education` is two lines inside a 30px-padded band (~130px tall).** The observer is not
  behaviorally equivalent to the prototype's "last section whose top ≤ scrollTop + 90" heuristic; a
  section shorter than the effective band can fail to become active. Verify Education highlights on
  a real scroll before considering this task done. If `rootMargin` cannot be tuned to cover it,
  fall back to an explicit scroll-position calculation over the section offsets — that is an
  acceptable outcome, not a failure.
- Default `activeSection` to `'about'`.

### Task 5.4 — Click: set active, scroll, update the URL
1. Set `activeSection` **immediately** — do not wait for the observer to confirm.
2. `element.scrollIntoView({ behavior, block: 'start' })`. The landing offset comes from the
   section's `scrollMarginTop` (Phase 4), so no manual offset math is needed.
3. Honor reduced motion: `behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth'`.
4. Update the URL with **router-aware** navigation — `navigate(path, { replace: true })`. Raw
   `history.replaceState` fights react-router's own history management and desyncs its location
   state.

⚠️ **There is no `/about` route.** `navigate('/' + id)` would push `/about`, which is registered
neither in `App.tsx` nor in `src/startup.rs` — a refresh there returns a real 404. Map explicitly:

```ts
const SECTION_PATHS = {
  about: '/',
  skills: '/skills',
  experience: '/experience',
  education: '/education',
  portfolio: '/portfolio',
} as const;
```

(If you prefer hash anchors — `/#skills` — that also works and keeps one path, but then the deep-link
URLs the sitemap and server routes already support stop matching what the nav produces. Path-based is
the option Decision 1 assumes.)

### Task 5.5 — Initial scroll on direct deep links
On mount, resolve `location.pathname` to a section id and scroll there once layout has settled
(after fonts/images affect offsets — `requestAnimationFrame` or a layout effect, not a bare
`useEffect` with a magic timeout):

| path | section |
|---|---|
| `/` | `about` (no scroll — already at top) |
| `/skills` `/experience` `/education` `/portfolio` | matching id |
| `/open-source` `/projects` | **`portfolio`** |

The legacy paths matter: Phase 3 removed their `<Navigate to="/portfolio">` redirects, so this is
now the only thing that makes them land in the right place.

### Task 5.6 — Do not port `getSelectedOption()`
`ResponsiveAppBar.tsx:70-75` derives the active label by slicing the pathname — a fragile string
round-trip. Note also that its `color: 'primary'` (line 202) is an **inert** `sx` value, not a
palette leaf, so today's "active" state is actually conveyed only by underline and weight. The new
nav's active styling is explicit; deleting the old helper is Phase 7.

---

## Files Created/Changed
- `ui/src/components/nav/SectionNav.tsx` *(new)*
- `ui/src/App.tsx` (renders `SectionNav` between `Masthead` and the section stack)

---

## Verification Steps
1. Scroll: `SectionNav` sticks flush under `ResponsiveAppBar` at both breakpoints, and stays *under*
   it in z-order.
2. Active item updates through all five sections — **including Education** (the known-risky one).
3. Scroll up past 16px and back down: the scrolled background/shadow does not flicker
   (`disableHysteresis`).
4. Click each item: smooth scroll, heading lands clear of both bars, active state changes instantly.
5. With OS reduced-motion enabled, clicks jump instantly.
6. Load `localhost:8080/skills` directly → 200, page auto-scrolls to Skills. Same for
   `/open-source` and `/projects` → Portfolio.
7. After clicking through the nav, press Back/Forward — react-router's location stays in sync and no
   navigation 404s.
8. Keyboard: tab through the nav strip; focus is visible and anchors land below both bars.
