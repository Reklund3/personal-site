import React from 'react';
import { Box, Container, Typography, Link, Divider } from '@mui/material';
import GitHubIcon from '@mui/icons-material/GitHub';

export const Footer: React.FC = () => {
    const currentYear = new Date().getFullYear();

    return (
        <Box
            component="footer"
            sx={{
                mt: 'auto',
                py: 3,
                px: 2,
                backgroundColor: (theme) =>
                    theme.palette.mode === 'light'
                        ? theme.palette.grey[200]
                        : theme.palette.grey[800],
            }}
        >
            <Container maxWidth="lg">
                <Divider sx={{ mb: 2 }} />

                {/* Attribution Section */}
                <Box sx={{ mb: 2 }}>
                    <Typography variant="body2" color="text.secondary" align="center">
                        Built with{' '}
                        <Link href="https://www.rust-lang.org/" target="_blank" rel="noopener noreferrer">
                            Rust
                        </Link>
                        {' '}(Actix Web),{' '}
                        <Link href="https://react.dev/" target="_blank" rel="noopener noreferrer">
                            React
                        </Link>
                        , and{' '}
                        <Link href="https://www.typescriptlang.org/" target="_blank" rel="noopener noreferrer">
                            TypeScript
                        </Link>
                    </Typography>
                    <Typography variant="body2" color="text.secondary" align="center" sx={{ mt: 1 }}>
                        Code assistance provided by{' '}
                        <Link href="https://claude.ai/code" target="_blank" rel="noopener noreferrer">
                            Claude Code
                        </Link>
                    </Typography>
                </Box>

                {/* Copyright and License */}
                <Box sx={{ mb: 2 }}>
                    <Typography variant="body2" color="text.secondary" align="center">
                        © {currentYear} Robert Eklund. All rights reserved.
                    </Typography>
                    <Typography variant="body2" color="text.secondary" align="center" sx={{ mt: 0.5 }}>
                        Content licensed under{' '}
                        <Link
                            href="https://creativecommons.org/licenses/by/4.0/"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            CC BY 4.0
                        </Link>
                    </Typography>
                </Box>

                {/* Source Code Link */}
                <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', gap: 1 }}>
                    <GitHubIcon fontSize="small" color="action" />
                    <Typography variant="body2" color="text.secondary">
                        <Link
                            href="https://github.com/Reklund3/personal-site"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            View Source Code
                        </Link>
                        {' '}(MIT License)
                    </Typography>
                </Box>

                {/* Privacy Notice */}
                <Typography variant="caption" color="text.secondary" align="center" display="block" sx={{ mt: 2 }}>
                    This site does not use cookies for tracking or analytics.
                </Typography>
            </Container>
        </Box>
    );
};
