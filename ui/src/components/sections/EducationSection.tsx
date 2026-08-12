import React from 'react';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import { useAppBarHeight } from '../../context/AppBarHeightContext';
import { useTheme } from '@mui/material/styles';
import { CONTENT } from '../../content';

export default function EducationSection() {
  const { appBarHeight } = useAppBarHeight();
  const theme = useTheme();

  const scrollMarginTop = appBarHeight + 64;

  return (
    <Box
      component="section"
      id="education"
      sx={{
        bgcolor: theme.palette.surface?.alt,
        scrollMarginTop: `${scrollMarginTop}px`,
      }}
    >
      <Container
        maxWidth="lg"
        sx={{
          px: { xs: '26px', md: '60px' },
          py: '30px',
        }}
      >
        {/* Eyebrow */}
        <Typography
          variant="eyebrow"
          color="primary.main"
          sx={{
            mb: '14px',
            textAlign: 'center',
          }}
        >
          EDUCATION
        </Typography>

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
      </Container>
    </Box>
  );
}
