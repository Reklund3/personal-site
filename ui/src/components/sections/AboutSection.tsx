import React from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import Section from './Section';
import { CONTENT } from '../../content';

export default function AboutSection() {
  return (
    <Section id="about" eyebrow="ABOUT" band="default" eyebrowGap={14}>
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
            <List
              disablePadding
              sx={{
                mb: '8px',
              }}
            >
              {section.bullets.map((bullet, idx) => (
                <ListItem
                  key={idx}
                  disableGutters
                  sx={{
                    fontSize: 12.5,
                    lineHeight: 1.65,
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
                    <strong style={{ color: 'rgba(255,255,255,.88)', marginRight: '0.25em' }}>
                      {bullet.label}:{' '}
                    </strong>
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
