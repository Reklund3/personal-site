/**
 * Deterministic chip colors for the Skills section.
 *
 * Every chip — technology and soft skill alike — derives its color from its own
 * name, so "Jira" is the same color on every render, in every category, forever.
 * There are no per-item color fields, no lookup table, and no exceptions: the
 * name is the only input.
 *
 * ## Why saturation and lightness are fixed
 *
 * The chips are outlined, so the color is used as TEXT and BORDER against the
 * card rather than as a fill. That is the harder contrast problem — a swatch
 * that reads comfortably with white type on top of it is often unreadable as
 * type itself.
 *
 * Rather than correct per hue, the values below were solved once: at S 90% /
 * L 75%, over a 12%-alpha tint of the same hue, every one of the 360 hues clears
 * WCAG AA (4.5:1), with the worst at 4.54:1. So the hash is free to land
 * anywhere on the circle and the result is legible by construction — no
 * clamping, no special cases, no verification when a skill is added.
 *
 * Saturation and lightness are coupled, not independent: pushing saturation up
 * *forces* lightness up to hold the contrast floor. That is why these read as
 * bright neon on dark rather than as deep, saturated color — on a dark surface
 * those are the same knob. The blues around hue 240 are the binding constraint
 * and fail first, so lowering L or raising the tint alpha breaks the guarantee.
 *
 * The tint alpha matters to the math because the label sits ON the tint, not on
 * the bare card: raising it lifts the background toward the text and eats the
 * margin. 12% was chosen as the most fill the palette can carry at this
 * lightness.
 *
 * The card is Paper at elevation 1, which in MUI's dark mode composites 5% white
 * over #121212 — i.e. #1e1e1e, matching the design handoff.
 */

const SATURATION = 90;
/** Lowest lightness at which all 360 hues clear 4.5:1 over the tinted pill. */
const LIGHTNESS = 75;

/** Pill fill. Kept low so the chip reads as tinted glass, not a solid block. */
const FILL_ALPHA = 0.12;
/** Outline — brighter than the fill so the pill keeps a defined edge. */
const BORDER_ALPHA = 0.55;

export interface SkillChipColor {
  /** Label color, and the hue everything else is derived from. */
  fg: string;
  /** Translucent fill behind the label. */
  fill: string;
  /** Outline color. */
  border: string;
}

/**
 * djb2-style string hash. The only requirements are determinism and a reasonable
 * spread of short labels; it is not used for anything security-sensitive.
 */
function hashName(name: string): number {
  let hash = 0;
  for (let i = 0; i < name.length; i += 1) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
    // Coerce back to a 32-bit int so long names cannot drift into float space.
    hash |= 0;
  }
  return Math.abs(hash);
}

export function skillChipColor(name: string): SkillChipColor {
  const hue = hashName(name) % 360;
  const at = (alpha: number) => `hsla(${hue}, ${SATURATION}%, ${LIGHTNESS}%, ${alpha})`;
  return {
    fg: `hsl(${hue}, ${SATURATION}%, ${LIGHTNESS}%)`,
    fill: at(FILL_ALPHA),
    border: at(BORDER_ALPHA),
  };
}
