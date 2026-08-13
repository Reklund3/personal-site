import React, { ReactNode } from 'react';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import { useSectionNavHeight } from '../../context/SectionNavHeightContext';
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
  const { navHeight } = useSectionNavHeight();
  const theme = useTheme();

  // Determine background color based on band
  const bgColor = band === 'alt' ? theme.palette.surface?.alt : theme.palette.background.default;

  // Calculate scroll margin top: navHeight (sticky bar at top: 0)
  const scrollMarginTop = navHeight;

  return (
    <Box
      component="section"
      id={id}
      // A <section> is only exposed as a `region` landmark once it has an
      // accessible name, so without this the five main sections were invisible to
      // landmark navigation while the footer's columns — which do set it — were
      // not. Pointing at the eyebrow reuses the visible heading as the name.
      aria-labelledby={`${id}-heading`}
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
          id={`${id}-heading`}
          // `color="primary.main"` does NOT work here: on MUI 9 Typography matches
          // `color` against variants generated from bare palette keys, so a dotted
          // path silently resolves to nothing. Set it through sx instead.
          //
          // The `eyebrow` variant applies text-transform: uppercase, so callers
          // pass sentence case ("About", not "ABOUT"). Identical on screen, but
          // some screen readers spell out all-caps strings letter by letter.
          sx={{
            color: 'primary.main',
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
