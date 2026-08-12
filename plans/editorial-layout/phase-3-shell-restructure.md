# Phase 3: Shell Restructure & Layout Context

## Objective
Refactor the global layout shell (`ui/src/main.tsx`, `ui/src/App.tsx`, `ResponsiveAppBar.tsx`) to enable edge-to-edge section background bands, live AppBar height tracking via `ResizeObserver`, shared `ContactDialog` state, and a client route table that resolves every SPA path to the one-pager.

> **Task 3.2 is a prerequisite bug fix, not incidental redesign work.** The stale-height bug exists
> today; it is merely invisible because nothing depends on the value at a breakpoint boundary. Land
> it as its own commit (ideally its own PR) so it stays independently reviewable.

---

## Current shape (what you are changing)

- `main.tsx:16` — `AppLayout` owns `const [appBarHeight, setAppBarHeight] = useState(0)`.
- `main.tsx:24` — passes `onHeightMeasured={(height) => setAppBarHeight(height)}` down as a **prop**.
- `main.tsx:27-28` — consumes it as `marginTop: \`${appBarHeight}px\`` and applies `padding: 1`.
- `ResponsiveAppBar.tsx:28-33` — a one-shot `useLayoutEffect` + `offsetHeight`. No `ResizeObserver`,
  no resize listener, so the value **goes stale when the breakpoint is crossed by resizing**
  (desktop is a two-row block ~110–120px, mobile a single Toolbar ~64px).
- `App.tsx:53` — `<Container maxWidth="lg">` wraps everything.

---

## Detailed Tasks

### Task 3.1 — Create the AppBar height context (`ui/src/context/AppBarHeightContext.tsx`)

```tsx
import { createContext, useContext, useMemo, useState, type ReactNode } from 'react';

interface AppBarHeightValue {
  appBarHeight: number;
  setAppBarHeight: (height: number) => void;
}

const AppBarHeightContext = createContext<AppBarHeightValue>({
  appBarHeight: 0,
  setAppBarHeight: () => {},
});

export function AppBarHeightProvider({ children }: { children: ReactNode }) {
  const [appBarHeight, setAppBarHeight] = useState(0);
  // Memoize: an inline object literal changes identity every render and re-renders
  // every consumer, including the sticky nav on each scroll-driven parent update.
  const value = useMemo(() => ({ appBarHeight, setAppBarHeight }), [appBarHeight]);
  return <AppBarHeightContext.Provider value={value}>{children}</AppBarHeightContext.Provider>;
}

export const useAppBarHeight = () => useContext(AppBarHeightContext);
```

The provider must wrap `AppLayout` in `main.tsx` — **above** it, not inside — so `AppLayout` itself
can consume `appBarHeight` for its `marginTop`. Either nest it in the router element or wrap
`<RouterProvider>`.

### Task 3.2 — Replace the one-shot measurement with a `ResizeObserver`
In `ResponsiveAppBar`:
- **Delete the `onHeightMeasured` prop and the `ResponsiveAppBarProps` interface** (lines 22–24) —
  the component reads `setAppBarHeight` from context instead. Remove the prop at the `main.tsx:24`
  call site too; leaving it produces a silent no-op.
- Observe `appBarRef.current` with a `ResizeObserver`, writing every change into context. Disconnect
  on unmount.
- Guard against redundant writes (`if (h !== appBarHeight)`) — `ResizeObserver` fires on every layout
  pass and each `setState` re-renders the whole shell.

Incidental win: `ResponsiveAppBar` is wrapped in `memo()` (line 306), which the inline arrow at
`main.tsx:24` currently defeats on every parent render. Dropping the prop makes the memo real.

### Task 3.3 — Lift contact dialog state
`contactDialogOpen` is private to `ResponsiveAppBar` (`useState` at line 37, `<ContactDialog>` at
line 300). The masthead's "Contact Now" needs the same dialog. Create
`ui/src/context/ContactDialogContext.tsx` exposing `{ open, openDialog, closeDialog }`, render a
single `<ContactDialog dialogOpen={open} onClose={closeDialog} />` at the `AppLayout` level, and have
both CTAs call `openDialog`.

`ContactDialog`'s own API — `{ dialogOpen: boolean; onClose: () => void }` — does not change.

### Task 3.4 — Full-bleed bands
- Remove `<Container maxWidth="lg">` from `App.tsx:53` (keep the `Suspense`/`Routes` inside it).
- Remove `padding: 1` from `main.tsx:28`. **Keep `marginTop: appBarHeight`** — the AppBar is
  `position: fixed`, so the offset is still required.
- Bands are `<Box component="section">` at **`width: '100%'`**. ⚠️ **Never `100vw`** — `100vw`
  includes the vertical scrollbar's width and produces exactly the horizontal overflow that this
  phase's verification step 2 checks for.
- Each band gets an inner `<Container maxWidth="lg">` carrying the design inset (Phase 4 builds the
  reusable wrapper):
  - horizontal: **60px desktop / 26px mobile** (`px: { xs: '26px', md: '60px' }`)
  - vertical: **36px** for About / Skills / Experience / Portfolio, **30px** for Education, **22px**
    for the footer band

  ⚠️ These are the handoff's literal values (`padding: 36px 60px` / `36px 26px`). Do not substitute
  MUI spacing units — `py: 6, px: 6` is 48px/48px and visibly wrong against a design the handoff
  calls "high-fidelity … treated as final".

### Task 3.5 — Collapse the client route table (`App.tsx`)
Every SPA path must resolve to the **same** one-pager element, or Phase 7's deletion of the page
components breaks the router:

```tsx
<Routes>
  <Route path="/" element={<OnePager />} />
  <Route path="/skills" element={<OnePager />} />
  <Route path="/experience" element={<OnePager />} />
  <Route path="/education" element={<OnePager />} />
  <Route path="/portfolio" element={<OnePager />} />
  {/* legacy paths — land on the Portfolio anchor, no redirect */}
  <Route path="/open-source" element={<OnePager />} />
  <Route path="/projects" element={<OnePager />} />
  <Route path="*" element={<NotFound />} />
</Routes>
```

- The two `<Navigate to="/portfolio" replace />` redirects at `App.tsx:62-63` **go away** — under the
  one-pager they would bounce a deep link to a different URL for no reason. Phase 5 scrolls them to
  the Portfolio anchor instead.
- `<Route path="*" element={<NotFound />} />` **stays.** It is what keeps unknown client paths from
  silently rendering the one-pager, mirroring the server's deliberate lack of a `default_handler`.
- These seven paths must stay in lockstep with `src/startup.rs` (Phase 6).

### Task 3.6 — Strip section nav from `ResponsiveAppBar`
- Remove the desktop Row 2 `menuItemsTitles.map(...)` button strip (lines 195–218) and the mobile
  `Menu`'s `menuItemsTitles.map(...)` items (lines 259–263). Keep the mobile Resume `MenuItem`.
- Keep: logo/name/title, social links, Resume button, "Contact Now" CTA.
- `handleMenuItemClick` (48–55) and `getSelectedOption` (70–75) become dead — deleting them and
  `constants.ts` is Phase 7.
- Section navigation moves entirely to `SectionNav` (Phase 5).

---

## Files Changed
- `ui/src/context/AppBarHeightContext.tsx` *(new)*
- `ui/src/context/ContactDialogContext.tsx` *(new)*
- `ui/src/components/app-bar/ResponsiveAppBar.tsx`
- `ui/src/main.tsx`
- `ui/src/App.tsx`

---

## Verification Steps
1. Resize desktop (1240px) ↔ mobile (390px) **without reloading**; `appBarHeight` tracks live and
   content never slides under the AppBar. This is the regression the one-shot measurement causes and
   the reason Task 3.2 is a prerequisite.
2. Bands bleed to both screen edges with **no horizontal scrollbar** at any width.
3. "Contact Now" in the AppBar opens `ContactDialog`; only **one** dialog instance exists in the DOM.
4. `/open-source` and `/projects` stay on their own URL (no bounce to `/portfolio`) and return 200.
5. `/nonsense` still renders `NotFound`.
