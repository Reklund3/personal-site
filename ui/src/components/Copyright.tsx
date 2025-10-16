import { Box, Link, Typography, Stack } from "@mui/material";
import GitHubIcon from '@mui/icons-material/GitHub';
import LinkedInIcon from '@mui/icons-material/LinkedIn';
import React from "react";

function Copyright() {
    const currentYear = new Date().getFullYear();

    return (
        <Box
            component="footer"
            sx={{
                display: 'grid',
                gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, 1fr)', md: 'repeat(4, 1fr)' },
                gap: 3,
                pt: 3,
                pb: 2
            }}
        >
            {/* Column 1: Copyright & Legal */}
            <Box>
                <Typography variant="subtitle2" fontWeight={600} gutterBottom color="text.primary">
                    Legal
                </Typography>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                    © {currentYear} Robert Eklund
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block" gutterBottom>
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
            <Box>
                <Typography variant="subtitle2" fontWeight={600} gutterBottom color="text.primary">
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
            <Box>
                <Typography variant="subtitle2" fontWeight={600} gutterBottom color="text.primary">
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
            <Box>
                <Typography variant="subtitle2" fontWeight={600} gutterBottom color="text.primary">
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
    );
}

export default Copyright;