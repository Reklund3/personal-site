import React from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Section from './Section';
import { CONTENT } from '../../content';

/**
 * Experience zigzag timeline.
 *
 * Deliberately NOT built on @mui/lab's <Timeline>. The design's centre rule is a
 * single absolutely-positioned element spanning the full height of the section
 * (handoff markup line 459: `position:absolute;left:50%;top:0;bottom:0`), whereas
 * Timeline composes the rule out of one <TimelineConnector> per item — which leaves
 * a visible gap at every item boundary and cannot reach above the first entry or
 * below the last. TimelineSeparator also consumes width inside the flex row, so the
 * design's plain `justify-content` + `width:46%` cannot be expressed through it.
 * That is a structural mismatch, not a styling one; no specificity override fixes it.
 *
 * @mui/lab is still used for <Masonry> in PortfolioSection.
 */
export default function ExperienceSection() {
  return (
    <Section
      id="experience"
      eyebrow="EXPERIENCE"
      band="default"
      eyebrowGap={22}
      centerEyebrow={true}
    >
      <Box sx={{ position: 'relative' }}>
        {/* The continuous centre rule. Hidden below md, where the zigzag collapses. */}
        <Box
          aria-hidden="true"
          sx={{
            position: 'absolute',
            left: '50%',
            top: 0,
            bottom: 0,
            width: '2px',
            bgcolor: 'rgba(255,255,255,.15)',
            transform: 'translateX(-1px)',
            display: { xs: 'none', md: 'block' },
          }}
        />

        {/*
          A list, so assistive tech reads four entries rather than a run of loose
          paragraphs. DOM order stays chronological — the zigzag is presentational
          only, driven by index parity, so the reading order matches the mobile stack.
        */}
        <Box component="ul" sx={{ listStyle: 'none', p: 0, m: 0 }}>
          {CONTENT.experience.map((entry, index) => {
            // Mirrors the handoff's `jobsZigzag` exactly (markup line 962).
            const isEven = index % 2 === 0;
            return (
              <Box
                component="li"
                key={`${entry.company}-${entry.title}`}
                sx={{
                  display: 'flex',
                  justifyContent: {
                    xs: 'flex-start',
                    md: isEven ? 'flex-end' : 'flex-start',
                  },
                  mb: '26px',
                }}
              >
                <Box
                  sx={{
                    width: { xs: '100%', md: '46%' },
                    textAlign: { xs: 'left', md: isEven ? 'right' : 'left' },
                  }}
                >
                  <Typography
                    component="h3"
                    sx={{
                      fontSize: 14,
                      fontWeight: 700,
                      color: 'rgba(255,255,255,.9)',
                    }}
                  >
                    {entry.title}
                  </Typography>

                  <Typography
                    sx={{
                      fontSize: 11.5,
                      color: 'rgba(255,255,255,.55)',
                      mb: '8px',
                    }}
                  >
                    {entry.company} &middot; {entry.dates}
                  </Typography>

                  {/*
                    No literal "•" — the handoff renders these as plain lines
                    (markup line 466). listStyle:none keeps the list semantics
                    without a glyph a screen reader would announce.
                  */}
                  <Box component="ul" sx={{ listStyle: 'none', p: 0, m: 0 }}>
                    {entry.bullets.map((bullet) => (
                      <Typography
                        component="li"
                        key={bullet}
                        sx={{
                          fontSize: 12,
                          lineHeight: 1.6,
                          color: 'rgba(255,255,255,.72)',
                          mb: '4px',
                        }}
                      >
                        {bullet}
                      </Typography>
                    ))}
                  </Box>
                </Box>
              </Box>
            );
          })}
        </Box>
      </Box>
    </Section>
  );
}
