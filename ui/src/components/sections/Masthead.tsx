import React from 'react';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Avatar from '@mui/material/Avatar';
import { useContactDialog } from '../../context/ContactDialogContext';
import { CONTENT } from '../../content';

export default function Masthead() {
  const { openDialog } = useContactDialog();

  return (
    <Box
      sx={{
        background: 'radial-gradient(circle at 50% 0%, #1c2733, #121212 70%)',
        textAlign: 'center',
        pt: { xs: '48px', md: '48px' },
        pr: { xs: '26px', md: '60px' },
        pb: '34px',
        pl: { xs: '26px', md: '60px' },
      }}
    >
      <Container maxWidth="lg">
        {/*
          Avatar — the real headshot served by GET /headshot (src/routes/headshot.rs).

          The handoff drew this as a plain initials circle (markup line 399) only
          because the designer had no image asset; its README line 86 says to
          "reuse any existing avatar image asset if the current site has one",
          and the site does. The old ResponsiveAppBar was the sole consumer of
          /headshot and was deleted in 33953f1, which is what left the initials
          showing here.

          "RE" stays as the fallback: Avatar renders its children whenever the
          image is absent or fails to load, so the handoff's circle is exactly
          what a broken /headshot degrades to.

          Avatar already supplies borderRadius 50%, display:flex, centering and
          overflow:hidden, so only the deltas are set below.
        */}
        <Avatar
          src="/headshot"
          alt={CONTENT.profile.name}
          sx={{
            width: 88,
            height: 88,
            mx: 'auto',
            mb: '18px',
            // Fallback-only styling. When no image is showing, MUI adds its
            // `colorDefault` class, which paints grey[600] with `background.default`
            // text (Avatar.js:73-91) — not the handoff's #90caf9 / #062341. These
            // sx rules are inserted after the component styles, so they win.
            bgcolor: 'primary.main',
            color: 'primary.contrastText',
            fontSize: 30,
            fontWeight: 700,
          }}
        >
          RE
        </Avatar>

        {/* Name */}
        <Typography
          variant="h4"
          component="h1"
          sx={{
            fontWeight: 700,
            letterSpacing: '-0.5px',
            color: '#fff',
            mb: '6px',
          }}
        >
          {CONTENT.profile.name}
        </Typography>

        {/* Title */}
        <Typography
          sx={{
            fontSize: 15,
            color: 'rgba(255,255,255,.6)',
            mb: '16px',
          }}
        >
          {CONTENT.profile.title}
        </Typography>

        {/* Tagline */}
        <Typography
          sx={{
            fontSize: 13.5,
            lineHeight: 1.7,
            color: 'rgba(255,255,255,.75)',
            maxWidth: 420,
            mx: 'auto',
            mb: '22px',
          }}
        >
          {CONTENT.profile.tagline}
        </Typography>

        {/* Button row */}
        <Stack
          direction="row"
          spacing="10px"
          useFlexGap
          sx={{
            flexWrap: 'wrap',
            justifyContent: 'center',
          }}
        >
          {/* Contact Now button */}
          <Button
            variant="contained"
            onClick={openDialog}
            sx={{
              borderRadius: '20px',
              padding: '10px 22px',
              fontSize: 12.5,
              bgcolor: 'primary.main',
              color: 'primary.contrastText',
              '&:hover': {
                bgcolor: 'primary.main',
                opacity: 0.9,
              },
            }}
          >
            Contact Now
          </Button>

          {/* Resume button — live link to the /resume download */}
          <Button
            component="a"
            href="/resume"
            variant="outlined"
            sx={{
              borderRadius: '20px',
              padding: '10px 22px',
              fontSize: 12.5,
              // .35, not the handoff's .3: as an interactive control boundary this
              // is held to WCAG 1.4.11's 3:1, and .3 measures 2.67:1 against the
              // masthead gradient. .34 is the threshold; .35 leaves a margin.
              border: '1px solid rgba(255,255,255,.35)',
              color: 'rgba(255,255,255,.75)',
              '&:hover': {
                border: '1px solid rgba(255,255,255,.5)',
              },
            }}
          >
            ↓ Resume
          </Button>
        </Stack>
      </Container>
    </Box>
  );
}
