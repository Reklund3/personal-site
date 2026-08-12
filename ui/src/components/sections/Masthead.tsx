import React from 'react';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
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
        {/* Avatar */}
        <Box
          sx={{
            width: 88,
            height: 88,
            borderRadius: '50%',
            bgcolor: 'primary.main',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            mx: 'auto',
            mb: '18px',
            fontSize: 30,
            fontWeight: 700,
            color: 'primary.contrastText',
          }}
        >
          RE
        </Box>

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
              textTransform: 'none',
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
              border: '1px solid rgba(255,255,255,.3)',
              color: 'rgba(255,255,255,.75)',
              textTransform: 'none',
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
