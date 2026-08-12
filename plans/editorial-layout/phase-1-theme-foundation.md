# Phase 1: Theme Foundation & MUI Configuration

## Objective
Establish the theme foundation in MUI for the Editorial One-Pager design, forcing dark mode, defining the one net-new palette surface token, setting component default styles, and typing custom theme tokens.

`ui/src/theme.tsx` is currently 23 lines and almost entirely commented out — `cssVariables: true` and `colorSchemes: { dark: true }` are the only live options.

---

## Detailed Tasks

### Task 1.1 — Actually force dark mode in `ui/src/theme.tsx`
⚠️ **`colorSchemes: { dark: true }` is already there and does *not* force dark.** Verified against the
installed MUI **9.3.1**: that config produces `colorSchemes: ['light','dark']`,
`defaultColorScheme: 'light'`, `colorSchemeSelector: 'media'`, `palette.mode: 'light'` — i.e. light
mode is live and follows the visitor's OS setting. Since the handoff supplies dark values only, a
visitor on a light OS gets an unspecified, unstyled-looking page today.

Add `defaultColorScheme: 'dark'`:

```typescript
const theme = createTheme({
    cssVariables: true,
    colorSchemes: { dark: true },
    defaultColorScheme: 'dark',   // ← collapses to a single dark scheme
    // …palette + components below
});
```

Verified results of the candidate configs (all re-run against the installed **9.3.1**; the numbers
are unchanged from 6.5.0, so this task survived the upgrade intact):

| config | resulting schemes | `palette.mode` |
|---|---|---|
| `colorSchemes: { dark: true }` *(today)* | `light`, `dark` | **`light`** |
| `+ defaultColorScheme: 'dark'` | `dark` only | `dark` ✅ |
| `colorSchemes: { light: false, dark: true }` | — | **throws** `colorSchemes.light … missing or invalid` |
| `palette: { mode: 'dark' }` | `dark` only | `dark` (also valid) |

Do **not** set `primary.main` — dark `primary.main` is already `blue[200]` = `#90caf9`, exactly the
design accent. Do **not** set `background.default` — it is already `#121212`.

### Task 1.2 — Override `primary.contrastText`
`#062341` is the only palette override strictly required. MUI computes `primary.contrastText` as
`rgba(0, 0, 0, 0.87)` for `#90caf9` (verified), so the design's text-on-accent value must be set
explicitly:

```typescript
colorSchemes: { dark: { palette: { primary: { contrastText: '#062341' } } } },
```

⚠️ `#062341` is 9.09:1 on the `#90caf9` accent (safe) but only **1.18:1 on `#121212`** — usable on
the accent only, never as body text.

### Task 1.3 — Add the net-new surface token (`#171717`) & module augmentation
`#171717` is genuinely bespoke: zero occurrences in MUI and unreachable via any of the 25 elevation
steps. It is the one token this design actually needs.

- Extend MUI's `Palette` / `PaletteOptions` (in `theme.tsx` or a `theme.d.ts`):
  ```typescript
  declare module '@mui/material/styles' {
    interface Palette {
      surface: { alt: string };
    }
    interface PaletteOptions {
      surface?: { alt?: string };
    }
  }
  ```
- Add `surface: { alt: '#171717' }` to the dark scheme's palette.
- ⚠️ **Do not add tokens for `#121212` or `#1e1e1e`.** `#121212` is already `background.default`.
  `#1e1e1e` is not a token at all — it is the *rendered* result of MUI's dark elevation overlay on a
  default `<Paper elevation={1}>` (`getOverlayAlpha(1) = 0.051` composited over `#121212` →
  `rgb(30,30,30)`, verified). Setting `background.paper: '#1e1e1e'` stacks the overlay on top and
  yields ~`#292929`. Cards get the design value for free.

### Task 1.4 — Component style overrides
Design radii: cards **10px**, chips **12px**, masthead pills **20px**, soft-skill pills **14px**
(`shape.borderRadius` default is 4).

```typescript
components: {
  MuiCard: { styleOverrides: { root: { borderRadius: 10 } } },
  MuiChip: { styleOverrides: { root: { borderRadius: 12, fontWeight: 500 } } },
}
```

⚠️ **Do not put `borderRadius: 20` on `MuiButton` globally.** It is the masthead pill radius, not a
site-wide value — a global override also restyles the AppBar's "Contact Now", the mobile menu, and
every `ContactDialog` button. Scope it with `sx` on the two masthead pills instead (Phase 4).

⚠️ Likewise, the global `MuiChip` radius of 12 is the *brand chip* value; the soft-skill pills are
**14px** outlined. Override those per-instance in Phase 4.

### Task 1.5 — Add an `eyebrow` typography variant
The design's eyebrow (11.5px / weight 700 / 1.5px letter-spacing / uppercase) does not match MUI's
`overline` variant (12px / weight 400 / `0.08333em`). Rather than re-applying `sx` at five call
sites, register it as a **custom typography variant** — the documented MUI pattern, in four parts:

```ts
// 1. theme.typography
eyebrow: {
  fontSize: 11.5, fontWeight: 700, letterSpacing: '1.5px',
  textTransform: 'uppercase', lineHeight: 1,
},
// 2. default semantic element — renders <h2> instead of <span>
components: {
  MuiTypography: { defaultProps: { variantMapping: { eyebrow: 'h2' } } },
},
```
```ts
// 3. TypeScript augmentation
declare module '@mui/material/styles' {
  interface TypographyVariants { eyebrow: React.CSSProperties }
  interface TypographyVariantsOptions { eyebrow?: React.CSSProperties }
}
declare module '@mui/material/Typography' {
  interface TypographyPropsVariantOverrides { eyebrow: true }
}
```

Then `<Typography variant="eyebrow">` in Phase 4 carries both the styling **and** the `<h2>` the
heading-outline requirement needs, with no per-section `sx`. Only `color` and `mb` stay as props,
since those vary.

### Task 1.6 — Skill chip brand colors stay **content**, not theme keys
Per-technology colors are data, not design tokens. **Do not create a separate
`ui/src/constants/brandColors.ts`** — it would fork the source of truth against Phase 2's content
model, which already carries `color` on every skill item (the prototype's `CONTENT.skills.categories[].items[]`
is `{ name, color }`).

Apply the two reconciled values in the Phase 2 content file:
- `JetBrains Suite`: `#000000` → **`#5b5b5b`**
- `Miro`: `#FFD02F` → **`#c9a800`**

The other 31 chips match the repo already.

---

## Files Changed
- `ui/src/theme.tsx`
- `ui/src/theme.d.ts` *(optional — module augmentation can live in `theme.tsx`)*

---

## Verification Steps
1. `cd ui && npm run build` — zero TypeScript errors with the palette augmentation.
2. In the browser with the OS set to **light** mode, the site still renders dark. (This is the
   regression Task 1.1 exists to prevent; it is invisible if you only ever test on a dark OS.)
3. DevTools: a default `<Paper elevation={1}>` computes to `rgb(30, 30, 30)` — **not** `#292929`,
   which would mean `background.paper` was wrongly overridden.
