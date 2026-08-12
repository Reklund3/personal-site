import React, { ReactNode } from 'react';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import { useAppBarHeight } from '../../context/AppBarHeightContext';
import { useTheme } from '@mui/material/styles';

interface SectionProps {
  /** Section identifier for anchor linking */
  id: string;
  /** Eyebrow text (rendered as h2 via custom variant) */
  eyebrow: string;
  /** Band background: 'default' (#121212) or 'alt' (#171717) */
  band: 'default' | 'alt';
  /** Gap between eyebrow and content (in pixels) */
  eyebrowGap: number;
  /** Optionally center the eyebrow */
  centerEyebrow?: boolean;
  /** Section content */
  children: ReactNode;
}

export default function Section({
  id,
  eyebrow,
  band,
  eyebrowGap,
  centerEyebrow,
  children,
}: SectionProps) {
  const { appBarHeight } = useAppBarHeight();
  const theme = useTheme();

  // Determine background color based on band
  const bgColor = band === 'alt' ? theme.palette.surface?.alt : theme.palette.background.default;

  // Calculate scroll margin top: appBarHeight + estimated nav height (64px for Phase 5)
  // For now, just appBarHeight; Phase 5 will add the sticky nav height
  const scrollMarginTop = appBarHeight + 64;

  return (
    <Box
      component="section"
      id={id}
      sx={{
        bgcolor: bgColor,
        scrollMarginTop: `${scrollMarginTop}px`,
      }}
    >
      <Container
        maxWidth="lg"
        sx={{
          px: { xs: '26px', md: '60px' },
          py: '36px',
        }}
      >
        <Typography
          variant="eyebrow"
          color="primary.main"
          sx={{
            mb: `${eyebrowGap}px`,
            textAlign: centerEyebrow ? 'center' : 'left',
          }}
        >
          {eyebrow}
        </Typography>
        {children}
      </Container>
    </Box>
  );
}
