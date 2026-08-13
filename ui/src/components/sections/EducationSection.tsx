import React from 'react';
import Typography from '@mui/material/Typography';
import Section from './Section';
import { CONTENT } from '../../content';

export default function EducationSection() {
  return (
    <Section id="education" eyebrow="Education" band="alt" eyebrowGap={14} centerEyebrow={true}>
      {/* Education lines */}
      {CONTENT.education.map((line, idx) => (
        <Typography
          key={idx}
          sx={{
            fontSize: 13,
            lineHeight: 1.7,
            color: 'rgba(255,255,255,.75)',
            mb: '6px',
            textAlign: 'center',
          }}
        >
          {line}
        </Typography>
      ))}
    </Section>
  );
}
