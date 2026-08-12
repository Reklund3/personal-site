import { Box, Link, Typography, Stack } from "@mui/material";
import GitHubIcon from '@mui/icons-material/GitHub';
import LinkedInIcon from '@mui/icons-material/LinkedIn';
import React from "react";

function Copyright() {
    const currentYear = new Date().getFullYear();

    return (
        <Box
            sx={{
                display: 'grid',
                gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, 1fr)', md: 'repeat(4, 1fr)' },
                gap: 3,
                pt: 3,
                pb: 2
            }}
        >
            {/* Column 1: Copyright & Legal */}
            <Box component="section" aria-labelledby="footer-legal">
                <Typography
                    id="footer-legal"
                    component="h2"
                    variant="subtitle2"
                    gutterBottom
                    sx={{
                        fontWeight: 600,
                        color: "text.primary"
                    }}>
                    Legal
                </Typography>
                <Typography variant="body2" gutterBottom sx={{
                    color: "text.secondary"
                }}>
                    © {currentYear} Robert Eklund
                </Typography>
                <Typography
                    variant="caption"
                    gutterBottom
                    sx={{
                        color: "text.secondary",
                        display: "block"
                    }}>
                    All rights reserved.
                </Typography>
                <Typography
                    variant="caption"
                    sx={{
                        color: "text.secondary",
                        fontStyle: 'italic',
                        display: 'block',
                        mt: 1
                    }}>
                    No commercial AI/ML training permitted.
                </Typography>
            </Box>

            {/* Column 2: Technology Stack */}
            <Box component="section" aria-labelledby="footer-built-with">
                <Typography
                    id="footer-built-with"
                    component="h2"
                    variant="subtitle2"
                    gutterBottom
                    sx={{
                        fontWeight: 600,
                        color: "text.primary"
                    }}>
                    Built With
                </Typography>
                <Stack spacing={0.5}>
                    <Link
                        href="https://www.rust-lang.org/"
                        target="_blank"
                        rel="noopener noreferrer"
                        variant="caption"
                        underline="hover"
                        sx={{
                            color: "text.secondary"
                        }}
                    >
                        Rust
                    </Link>
                    <Link
                        href="https://react.dev/"
                        target="_blank"
                        rel="noopener noreferrer"
                        variant="caption"
                        underline="hover"
                        sx={{
                            color: "text.secondary"
                        }}
                    >
                        React
                    </Link>
                    <Link
                        href="https://www.typescriptlang.org/"
                        target="_blank"
                        rel="noopener noreferrer"
                        variant="caption"
                        underline="hover"
                        sx={{
                            color: "text.secondary"
                        }}
                    >
                        TypeScript
                    </Link>
                </Stack>
            </Box>

            {/* Column 3: Licensing & Links */}
            <Box component="section" aria-labelledby="footer-resources">
                <Typography
                    id="footer-resources"
                    component="h2"
                    variant="subtitle2"
                    gutterBottom
                    sx={{
                        fontWeight: 600,
                        color: "text.primary"
                    }}>
                    Resources
                </Typography>
                <Stack spacing={0.5}>
                    <Link
                        href="https://creativecommons.org/licenses/by-nc-sa/4.0/"
                        target="_blank"
                        rel="noopener noreferrer"
                        variant="caption"
                        underline="hover"
                        aria-label="Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License"
                        sx={{
                            color: "text.secondary"
                        }}
                    >
                        Content License (CC BY-NC-SA 4.0)
                    </Link>
                    <Link
                        href="https://github.com/Reklund3/personal-site"
                        target="_blank"
                        rel="noopener noreferrer"
                        variant="caption"
                        underline="hover"
                        aria-label="View source code on GitHub"
                        sx={{
                            color: "text.secondary"
                        }}
                    >
                        Source Code
                    </Link>
                </Stack>
            </Box>

            {/* Column 4: Social/Connect */}
            <Box component="section" aria-labelledby="footer-connect">
                <Typography
                    id="footer-connect"
                    component="h2"
                    variant="subtitle2"
                    gutterBottom
                    sx={{
                        fontWeight: 600,
                        color: "text.primary"
                    }}>
                    Connect
                </Typography>
                <Stack spacing={0.5}>
                    <Link
                        href="https://github.com/Reklund3"
                        target="_blank"
                        rel="noopener noreferrer"
                        variant="caption"
                        underline="hover"
                        aria-label="GitHub Profile"
                        sx={{
                            color: "text.secondary",
                            display: 'flex',
                            alignItems: 'center',
                            gap: 0.5
                        }}>
                        <GitHubIcon fontSize="small" />
                        GitHub
                    </Link>
                    <Link
                        href="https://www.linkedin.com/in/robert-eklund-64302976/"
                        target="_blank"
                        rel="noopener noreferrer"
                        variant="caption"
                        underline="hover"
                        aria-label="LinkedIn Profile"
                        sx={{
                            color: "text.secondary",
                            display: 'flex',
                            alignItems: 'center',
                            gap: 0.5
                        }}>
                        <LinkedInIcon fontSize="small" />
                        LinkedIn
                    </Link>
                </Stack>
            </Box>
        </Box>
    );
}

export default Copyright;