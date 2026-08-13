import { Box, Container, Link, Typography, Stack } from "@mui/material";
import GitHubIcon from '@mui/icons-material/GitHub';
import LinkedInIcon from '@mui/icons-material/LinkedIn';
import React from "react";
import { useTheme } from '@mui/material/styles';

function Copyright() {
    const currentYear = new Date().getFullYear();
    const theme = useTheme();

    return (
        <Box
            sx={{
                bgcolor: theme.palette.surface?.alt,
                borderTop: '1px solid rgba(255,255,255,.1)',
                px: { xs: '26px', md: '60px' },
                py: 4,
            }}
        >
            <Container maxWidth="lg" disableGutters>
                <Box
                    sx={{
                        display: 'grid',
                        gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, 1fr)', md: 'repeat(4, 1fr)' },
                        gap: 3,
                    }}
                >
                    {/* Column 1: Copyright & Legal */}
                    <Box component="section" aria-labelledby="footer-legal">
                        <Typography id="footer-legal" component="h2" variant="subtitle2" gutterBottom color="text.primary" sx={{ fontWeight: 600 }}>
                            Legal
                        </Typography>
                        <Typography variant="body2" color="text.secondary" gutterBottom>
                            © {currentYear} Robert Eklund
                        </Typography>
                        <Typography variant="caption" color="text.secondary" gutterBottom sx={{ display: 'block' }}>
                            All rights reserved.
                        </Typography>
                        <Typography
                            variant="caption"
                            color="text.secondary"
                            sx={{
                                fontStyle: 'italic',
                                display: 'block',
                                mt: 1
                            }}
                        >
                            No commercial AI/ML training permitted.
                        </Typography>
                    </Box>

                    {/* Column 2: Technology Stack */}
                    <Box component="section" aria-labelledby="footer-built-with">
                        <Typography id="footer-built-with" component="h2" variant="subtitle2" gutterBottom color="text.primary" sx={{ fontWeight: 600 }}>
                            Built With
                        </Typography>
                        <Stack spacing={0.5}>
                            <Link
                                href="https://www.rust-lang.org/"
                                target="_blank"
                                rel="noopener noreferrer"
                                variant="caption"
                                color="text.secondary"
                                underline="hover"
                            >
                                Rust
                            </Link>
                            <Link
                                href="https://react.dev/"
                                target="_blank"
                                rel="noopener noreferrer"
                                variant="caption"
                                color="text.secondary"
                                underline="hover"
                            >
                                React
                            </Link>
                            <Link
                                href="https://www.typescriptlang.org/"
                                target="_blank"
                                rel="noopener noreferrer"
                                variant="caption"
                                color="text.secondary"
                                underline="hover"
                            >
                                TypeScript
                            </Link>
                        </Stack>
                    </Box>

                    {/* Column 3: Licensing & Links */}
                    <Box component="section" aria-labelledby="footer-resources">
                        <Typography id="footer-resources" component="h2" variant="subtitle2" gutterBottom color="text.primary" sx={{ fontWeight: 600 }}>
                            Resources
                        </Typography>
                        <Stack spacing={0.5}>
                            <Link
                                href="https://creativecommons.org/licenses/by-nc-sa/4.0/"
                                target="_blank"
                                rel="noopener noreferrer"
                                variant="caption"
                                color="text.secondary"
                                underline="hover"
                                aria-label="Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License"
                            >
                                Content License (CC BY-NC-SA 4.0)
                            </Link>
                            <Link
                                href="https://github.com/Reklund3/personal-site"
                                target="_blank"
                                rel="noopener noreferrer"
                                variant="caption"
                                color="text.secondary"
                                underline="hover"
                                aria-label="View source code on GitHub"
                            >
                                Source Code
                            </Link>
                        </Stack>
                    </Box>

                    {/* Column 4: Social/Connect */}
                    <Box component="section" aria-labelledby="footer-connect">
                        <Typography id="footer-connect" component="h2" variant="subtitle2" gutterBottom color="text.primary" sx={{ fontWeight: 600 }}>
                            Connect
                        </Typography>
                        <Stack spacing={0.5}>
                            <Link
                                href="https://github.com/Reklund3"
                                target="_blank"
                                rel="noopener noreferrer"
                                variant="caption"
                                color="text.secondary"
                                underline="hover"
                                sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}
                                aria-label="GitHub Profile"
                            >
                                <GitHubIcon fontSize="small" />
                                GitHub
                            </Link>
                            <Link
                                href="https://www.linkedin.com/in/robert-eklund-64302976/"
                                target="_blank"
                                rel="noopener noreferrer"
                                variant="caption"
                                color="text.secondary"
                                underline="hover"
                                sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}
                                aria-label="LinkedIn Profile"
                            >
                                <LinkedInIcon fontSize="small" />
                                LinkedIn
                            </Link>
                        </Stack>
                    </Box>
                </Box>
            </Container>
        </Box>
    );
}

export default Copyright;
