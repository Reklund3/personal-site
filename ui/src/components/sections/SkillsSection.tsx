import React from 'react';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Section from './Section';
import { CONTENT } from '../../content';

export default function SkillsSection() {
  return (
    <Section id="skills" eyebrow="SKILLS" band="alt" eyebrowGap={18}>
      {/* Grid of skill categories */}
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', md: 'repeat(auto-fit, minmax(200px, 1fr))' },
          gap: '14px',
          mb: '16px',
        }}
      >
        {CONTENT.skills.categories.map((category) => (
          <Card key={category.label}>
            <Box sx={{ p: '16px' }}>
              {/* Category label */}
              <Typography
                sx={{
                  fontSize: 12.5,
                  fontWeight: 'bold',
                  color: 'rgba(255,255,255,.85)',
                  mb: '10px',
                }}
              >
                {category.label}
              </Typography>

              {/* Skills chips */}
              <Stack
                direction="row"
                useFlexGap
                spacing="6px"
                sx={{ flexWrap: 'wrap' }}
              >
                {category.items.map((item) => (
                  <Chip
                    key={item.name}
                    label={item.name}
                    size="small"
                    sx={{
                      bgcolor: item.color,
                      color: '#fff',
                      fontSize: 11,
                    }}
                  />
                ))}
              </Stack>
            </Box>
          </Card>
        ))}
      </Box>

      {/* Soft-skill pills */}
      <Stack
        direction="row"
        useFlexGap
        spacing="8px"
        sx={{
          flexWrap: 'wrap',
          mt: '16px',
        }}
      >
        {CONTENT.skills.soft.map((skill) => (
          <Chip
            key={skill}
            label={skill}
            variant="outlined"
            size="small"
            sx={{
              borderRadius: '14px',
              padding: '5px 12px',
              fontSize: 11.5,
              color: 'rgba(255,255,255,.8)',
              borderColor: 'rgba(255,255,255,.2)',
            }}
          />
        ))}
      </Stack>
    </Section>
  );
}
