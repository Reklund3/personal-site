import React from 'react';
import Timeline from '@mui/lab/Timeline';
import TimelineItem from '@mui/lab/TimelineItem';
import TimelineSeparator from '@mui/lab/TimelineSeparator';
import TimelineConnector from '@mui/lab/TimelineConnector';
import TimelineContent from '@mui/lab/TimelineContent';
import Typography from '@mui/material/Typography';
import { useTheme } from '@mui/material/styles';
import Section from './Section';
import { CONTENT } from '../../content';

export default function ExperienceSection() {
  const theme = useTheme();

  return (
    <Section
      id="experience"
      eyebrow="EXPERIENCE"
      band="default"
      eyebrowGap={22}
      centerEyebrow={true}
    >
      <Timeline
        position="alternate"
        sx={{
          p: 0,
          m: 0,
          // `&&&` is deliberate, not a typo. @mui/lab's own TimelineItem rules are
          // inserted AFTER this sx block, so anything that merely ties on specificity
          // loses. Each repetition of `&` adds one class's worth of weight:
          //   ours `& .MuiTimelineItem-root::before`            (0,2,1)
          //   lab  `&:not(:has(.MuiTimelineOppositeContent-root))::before` (0,2,1)  tie -> lab wins
          //   ours `&&& .MuiTimelineContent-root`               (0,4,0)
          //   lab  `&:nth-of-type(even) .MuiTimelineContent-root`(0,3,0)  we win
          // The text-align rule is the one that genuinely needs three; the rest use it
          // for consistency so nobody "tidies" one back down and silently breaks it.
          '&&& .MuiTimelineItem-root::before': {
            flex: '0 0 46%',
            p: 0,
          },
          '&&& .MuiTimelineContent-root': {
            flex: '0 0 46%',
            py: 0,
          },
          // Mobile collapse: one left-aligned column, no zigzag, no spacer.
          // Must out-specify lab's `:nth-of-type(even)` rules, which set both
          // flex-direction: row-reverse and text-align: right.
          [theme.breakpoints.down('md')]: {
            '&&& .MuiTimelineItem-root': {
              flexDirection: 'row',
            },
            '&&& .MuiTimelineItem-root::before': {
              display: 'none',
            },
            '&&& .MuiTimelineContent-root': {
              flex: 1,
              textAlign: 'left',
            },
          },
        }}
      >
        {CONTENT.experience.map((entry, index) => (
          <TimelineItem key={index}>
            <TimelineSeparator>
              <TimelineConnector
                sx={{
                  width: '2px',
                  bgcolor: 'rgba(255,255,255,.15)',
                }}
              />
            </TimelineSeparator>
            <TimelineContent>
              {/* Job title as h3 */}
              <Typography
                component="h3"
                sx={{
                  fontSize: 14,
                  fontWeight: 'bold',
                  color: 'rgba(255,255,255,.9)',
                }}
              >
                {entry.title}
              </Typography>

              {/* Company and dates */}
              <Typography
                sx={{
                  fontSize: 11.5,
                  color: 'rgba(255,255,255,.55)',
                  mb: '8px',
                }}
              >
                {entry.company} · {entry.dates}
              </Typography>

              {/* Bullets */}
              {entry.bullets.map((bullet, idx) => (
                <Typography
                  key={idx}
                  sx={{
                    fontSize: 12,
                    lineHeight: 1.6,
                    color: 'rgba(255,255,255,.72)',
                    mb: '4px',
                  }}
                >
                  • {bullet}
                </Typography>
              ))}
            </TimelineContent>
          </TimelineItem>
        ))}
      </Timeline>
    </Section>
  );
}
