import React from 'react';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Section from './Section';
import { CONTENT } from '../../content';
import { skillChipColor } from '../../utils/skillColor';

// Shared by every card in the grid, soft skills included, so the group heading
// is identical by construction rather than by two copies of the same values.
const CARD_LABEL_SX = {
  fontSize: 12.5,
  fontWeight: 'bold',
  color: 'rgba(255,255,255,.85)',
  mb: '10px',
} as const;

/**
 * One chip for the whole section — technologies and soft skills alike. The only
 * per-chip input is the label, which drives the color via `skillChipColor`.
 *
 * The geometry reproduces the handoff's pill (5px/12px padding, radius 14, 11.5px)
 * and has to fight two MUI Chip defaults to do it:
 *
 *  - the root is a fixed `height: 24` at size small, which makes vertical padding
 *    inert, so it is released to `auto`;
 *  - padding belongs on the LABEL slot, not the root. Chip's label carries its own
 *    7px inline padding, so setting `padding` on the root stacks on top of it and
 *    yields ~19px rather than the intended 12px.
 */
function SkillChip({ label }: { label: string }) {
  const { fg, fill, border } = skillChipColor(label);
  return (
    <Chip
      label={label}
      variant="outlined"
      size="small"
      sx={{
        height: 'auto',
        borderRadius: '14px',
        fontSize: 11.5,
        fontWeight: 600,
        color: fg,
        bgcolor: fill,
        borderColor: border,
        '& .MuiChip-label': { px: '12px', py: '5px' },
      }}
    />
  );
}

export default function SkillsSection() {
  return (
    <Section id="skills" eyebrow="Skills" band="alt" eyebrowGap={18}>
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', md: 'repeat(auto-fit, minmax(200px, 1fr))' },
          gap: '14px',
        }}
      >
        {/*
          Soft skills lead the grid as an ordinary cell, ahead of the technology
          categories.

          A deliberate deviation from the handoff, which leaves these as a bare
          flex row below the grid (markup line 449) with no card behind them —
          which read as dangling leftovers.
        */}
        <Card>
          <Box sx={{ p: '16px' }}>
            <Typography sx={CARD_LABEL_SX}>{CONTENT.skills.softLabel}</Typography>

            <Stack direction="row" useFlexGap spacing="8px" sx={{ flexWrap: 'wrap' }}>
              {CONTENT.skills.soft.map((skill) => (
                <SkillChip key={skill} label={skill} />
              ))}
            </Stack>
          </Box>
        </Card>

        {CONTENT.skills.categories.map((category) => (
          <Card key={category.label}>
            <Box sx={{ p: '16px' }}>
              <Typography sx={CARD_LABEL_SX}>{category.label}</Typography>

              <Stack direction="row" useFlexGap spacing="8px" sx={{ flexWrap: 'wrap' }}>
                {category.items.map((item) => (
                  <SkillChip key={item.name} label={item.name} />
                ))}
              </Stack>
            </Box>
          </Card>
        ))}
      </Box>
    </Section>
  );
}
