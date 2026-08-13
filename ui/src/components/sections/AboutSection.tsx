import React from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import Section from './Section';
import { CONTENT } from '../../content';

export default function AboutSection() {
  return (
    <Section id="about" eyebrow="About" band="default" eyebrowGap={14}>
      {/* Intro paragraph */}
      <Typography
        sx={{
          fontSize: 14,
          lineHeight: 1.8,
          color: 'rgba(255,255,255,.8)',
          mb: '18px',
        }}
      >
        {CONTENT.about.intro}
      </Typography>

      {/* Subsections */}
      {CONTENT.about.sections.map((section) => (
        <Box key={section.id}>
          {/* Subsection heading */}
          <Typography
            component="h3"
            sx={{
              fontSize: 13.5,
              fontWeight: 'bold',
              color: 'rgba(255,255,255,.88)',
              m: '16px 0 8px',
            }}
          >
            {section.heading}
          </Typography>

          {/* Intro paragraphs */}
          {section.intro.map((paragraph, idx) => (
            <Typography
              key={idx}
              sx={{
                fontSize: 13,
                lineHeight: 1.7,
                color: 'rgba(255,255,255,.72)',
                mb: '8px',
              }}
            >
              {paragraph}
            </Typography>
          ))}

          {/* Bullets */}
          {section.bullets.length > 0 && (
            <List disablePadding sx={{ mb: 0 }}>
              {section.bullets.map((bullet, idx) => (
                <ListItem
                  key={idx}
                  // BOTH are required. `disableGutters` only drops the horizontal
                  // 16px; without `disablePadding` each item keeps ListItem's default
                  // paddingTop/Bottom of 8px, which is 16px of extra space per bullet
                  // against the design's 5px margin.
                  disableGutters
                  disablePadding
                  sx={{
                    // ListItem defaults to `display: flex; align-items: center`, which
                    // makes the <strong> label and the text separate flex items — the
                    // text then cannot wrap underneath the label, and the two are
                    // vertically centred against each other. The design is ordinary
                    // inline text flow with a hanging dot, so force it back to block.
                    display: 'block',
                    fontSize: 12.5,
                    lineHeight: 1.65,
                    mb: '5px',
                    pl: '14px',
                    position: 'relative',
                    color: 'rgba(255,255,255,.72)',
                    '&::before': {
                      content: '""',
                      position: 'absolute',
                      left: 0,
                      top: '7px',
                      width: 4,
                      height: 4,
                      bgcolor: 'primary.main',
                      borderRadius: '50%',
                    },
                  }}
                >
                  {bullet.label && (
                    <Box component="strong" sx={{ color: 'rgba(255,255,255,.88)' }}>
                      {bullet.label}:{' '}
                    </Box>
                  )}
                  {bullet.text}
                </ListItem>
              ))}
            </List>
          )}

          {/* Outro paragraphs */}
          {section.outro.map((paragraph, idx) => (
            <Typography
              key={idx}
              sx={{
                fontSize: 13,
                lineHeight: 1.7,
                color: 'rgba(255,255,255,.72)',
                mb: '8px',
              }}
            >
              {paragraph}
            </Typography>
          ))}
        </Box>
      ))}
    </Section>
  );
}
