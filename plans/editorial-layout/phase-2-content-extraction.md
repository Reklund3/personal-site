# Phase 2: Content Extraction & Structured Data Model

## Objective
Extract all static portfolio content out of the individual page components into a strongly typed data file (`ui/src/content/index.ts`). This makes the section components data-driven and lets the SEO layer draw from the same source instead of a parallel table.

**Lift from the prototype's `CONTENT` object** (`Portfolio Layouts.dc.html` **lines 807–916**) rather
than re-scraping the five page components — the prototype has already restructured the copy for this
layout — then diff against the repo and reconcile the known deltas (Task 2.3).

> Requires Phase 0: the handoff `.dc.html` only exists inside `Alternative layout options.zip`.

---

## Detailed Tasks

### Task 2.1 — Define TypeScript interfaces (`ui/src/content/types.ts`)

These mirror the prototype's actual `CONTENT` shape. Earlier drafts of this doc invented fields
(`icon`, `tags`, `githubUrl`/`liveUrl`, a scalar `description`) that appear nowhere in the handoff,
and omitted fields the design actually renders (`subheader`, `linkLabel`, `profile`, `navItems`).

```typescript
export interface Bullet {
  /** Optional bold inline label rendered before the text, e.g. "Type Systems: …" */
  label?: string;
  text: string;
}

export interface AboutSubSection {
  id: string;
  heading: string;
  intro: string[];
  bullets: Bullet[];
  outro: string[];
}

export interface AboutContent {
  intro: string;
  sections: AboutSubSection[];
}

export interface SkillItem {
  name: string;
  /** Brand color for the chip background; white text is assumed. */
  color: string;
}

export interface SkillCategory {
  label: string;
  items: SkillItem[];
}

export interface SkillsContent {
  soft: string[];
  categories: SkillCategory[];
}

export interface ExperienceItem {
  title: string;
  company: string;
  /** Pre-formatted, e.g. "03/2022 – Present" (en dash). */
  dates: string;
  bullets: string[];
}

export interface PortfolioProject {
  title: string;
  /** Rendered as a subdued line under the title — open-source entries only. */
  subheader?: string;
  paragraphs: string[];
  link: string;
  /** Defaults to "View on GitHub" when absent, e.g. "View PR #28871". */
  linkLabel?: string;
}

export interface PortfolioContent {
  personal: PortfolioProject[];
  openSource: PortfolioProject[];
}

export interface NavItem {
  /** Matches the section element id and drives the scroll-spy. */
  id: 'about' | 'skills' | 'experience' | 'education' | 'portfolio';
  label: string;
}

export interface SiteContent {
  profile: {
    name: string;
    title: string;
    tagline: string;
    github: string;
    linkedin: string;
  };
  navItems: NavItem[];
  about: AboutContent;
  skills: SkillsContent;
  experience: ExperienceItem[];
  /** Two pre-composed lines; the design renders them as plain centered text. */
  education: string[];
  portfolio: PortfolioContent;
}
```

Notes on the shape:
- **`about.sections` uses `intro` → `bullets` → `outro`.** This ordering is load-bearing: a naive
  `{heading, paragraphs[], bullets[]}` model would wrongly hoist Technical Journey's closing
  paragraph above its bullets.
- **`about.intro`** is hardcoded in the prototype's markup but *is* present in `CONTENT` — keep it in
  the data model.
- **No `icon` field.** The repo's `Skills.tsx` renders a per-category MUI icon (`CodeIcon`,
  `CloudIcon`, `StorageIcon`, `BuildIcon`, `ConstructionIcon`); the editorial design drops icons
  entirely. If you decide to keep them, do **not** type it as `string` — a content module must not
  import JSX. Use a `iconKey: 'code' | 'cloud' | …` discriminant that the component maps to a
  component.
- **No `tags`.** Neither skills, experience, nor portfolio entries carry tags in the design.
- **Eyebrow strings** (`ABOUT`, `SKILLS`, …) are presentation, not content — they are a prop on the
  `Section` wrapper in Phase 4, not a field on every content type.

### Task 2.2 — Create `ui/src/content/index.ts`
Populate a single `export const CONTENT: SiteContent` from the prototype's object, cross-checked
against `Summary.tsx`, `Skills.tsx`, `Experience.tsx`, `Education.tsx`, `Portfolio.tsx`.

Decode the prototype's escapes: `–` is an en dash (date ranges), `—` an em dash (education
lines).

### Task 2.3 — Reconcile the known repo↔prototype deltas
Exactly three, all verified:

| | repo | prototype | resolution |
|---|---|---|---|
| `JetBrains Suite` chip | `#000000` | `#5b5b5b` | **take prototype** (black is near-invisible on `#1e1e1e`) |
| `Miro` chip | `#FFD02F` | `#c9a800` | **take prototype** (white-on-`#FFD02F` fails contrast) |
| Education line 1 | `Masters in…` | `Master's in…` | **take prototype** (apostrophe) |

Also note, but do **not** silently normalize: the Associate DevOps Engineer entry is
`bullets: ['Coming soon.']` in the prototype and lowercase `coming soon` in `Experience.tsx:80`.
That is a **content gap, not a typo** — on a single scrolling timeline it ships visibly. Fill or drop
the entry before launch (see README).

The other 31 skill chips match the repo exactly.

---

## Files Changed
- `ui/src/content/types.ts` *(new)*
- `ui/src/content/index.ts` *(new)*

---

## Verification Steps
1. `cd ui && npm run build` — type safety holds with no `any`.
2. Diff the rendered strings against the current pages: 6 About subsections, 6 skill categories +
   5 soft skills, 4 jobs, 2 education lines, 3 personal projects + 1 open-source entry.
3. Confirm the open-source entry retains both its `subheader` ("Factory Methods Enhancement") and its
   `linkLabel` ("View PR #28871") — the fields an invented `{description, githubUrl, tags}` model
   would have dropped.
