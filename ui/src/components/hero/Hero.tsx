import React from 'react';
import { Box, Typography, Button, Avatar, Tooltip, Fade } from '@mui/material';
import { Download, AccountCircle } from '@mui/icons-material';

export default function Hero() {
    return (
        <Fade in={true} timeout={1000}>
            <Box
                sx={{
                    display: 'flex',
                    flexDirection: { xs: 'column', md: 'row' },
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    gap: { xs: 3, md: 4 },
                    mb: 6,
                    p: { xs: 3, md: 4 },
                    background: (theme) =>
                        `linear-gradient(135deg, ${theme.palette.background.paper} 0%, ${theme.palette.background.default} 100%)`,
                    borderRadius: 2,
                    boxShadow: 3,
                }}
            >
                {/* Left side: Text content */}
                <Box
                    sx={{
                        flex: 1,
                        textAlign: { xs: 'center', md: 'left' },
                        order: { xs: 2, md: 1 },
                    }}
                >
                    <Typography
                        variant="h2"
                        component="h1"
                        sx={{
                            fontWeight: 700,
                            fontSize: { xs: '2rem', sm: '2.5rem', md: '3rem' },
                            mb: 1,
                        }}
                    >
                        Robert Eklund
                    </Typography>
                    <Typography
                        variant="h5"
                        component="h2"
                        sx={{
                            color: 'text.secondary',
                            fontWeight: 400,
                            fontSize: { xs: '1.1rem', sm: '1.3rem', md: '1.5rem' },
                            mb: 2,
                        }}
                    >
                        Software Engineer
                    </Typography>
                    <Typography
                        variant="body1"
                        sx={{
                            mb: 3,
                            lineHeight: 1.7,
                            maxWidth: { md: '600px' },
                            fontSize: { xs: '0.95rem', sm: '1rem' },
                        }}
                    >
                        Building scalable systems with functional programming principles.
                        Passionate about type safety, DevOps automation, and mentoring engineers.
                    </Typography>
                    <Tooltip
                        title="Resume download coming soon! Check back after the site upgrade is complete."
                        arrow
                    >
                        <span>
                            <Button
                                variant="contained"
                                startIcon={<Download />}
                                disabled
                                sx={{
                                    px: 3,
                                    py: 1,
                                    fontSize: { xs: '0.9rem', sm: '1rem' },
                                }}
                            >
                                Download Resume
                            </Button>
                        </span>
                    </Tooltip>
                </Box>

                {/* Right side: Avatar */}
                <Box
                    sx={{
                        order: { xs: 1, md: 2 },
                        display: 'flex',
                        justifyContent: 'center',
                        alignItems: 'center',
                    }}
                >
                    <Avatar
                        sx={{
                            width: { xs: 120, sm: 140, md: 160 },
                            height: { xs: 120, sm: 140, md: 160 },
                            bgcolor: 'primary.main',
                            fontSize: { xs: '2.5rem', sm: '3rem', md: '3.5rem' },
                        }}
                    >
                        <AccountCircle sx={{ fontSize: 'inherit' }} />
                    </Avatar>
                </Box>
            </Box>
        </Fade>
    );
}
