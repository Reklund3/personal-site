import { Box, Link, Typography } from "@mui/material";
import React from "react";

function Copyright() {
    const currentYear = new Date().getFullYear();

    return (
        <Box component="section" sx={{ display: 'flex', flexDirection: 'column', justifyContent: 'center', gap: 0.5 }}>
            {/* Copyright Notice */}
            <Typography
                variant="body2"
                align="center"
                sx={{
                    ml: 3,
                    color: 'text.secondary',
                }}
            >
                {'© '}
                {currentYear}
                {' Robert Eklund. All rights reserved.'}
            </Typography>

            {/* AI Training Restriction */}
            <Typography
                variant="caption"
                align="center"
                sx={{
                    ml: 3,
                    color: 'text.secondary',
                    fontStyle: 'italic',
                }}
            >
                This content may not be used for training artificial intelligence or machine learning models for commercial purposes.
            </Typography>

            {/* Attribution and Licensing */}
            <Typography
                variant="caption"
                align="center"
                sx={{
                    ml: 3,
                    color: 'text.secondary',
                }}
            >
                Built with{' '}
                <Link color="inherit" href="https://www.rust-lang.org/" target="_blank" rel="noopener noreferrer">
                    Rust
                </Link>
                {', '}
                <Link color="inherit" href="https://react.dev/" target="_blank" rel="noopener noreferrer">
                    React
                </Link>
                {', and '}
                <Link color="inherit" href="https://www.typescriptlang.org/" target="_blank" rel="noopener noreferrer">
                    TypeScript
                </Link>
            </Typography>

            {/* License Information */}
            <Typography
                variant="caption"
                align="center"
                sx={{
                    ml: 3,
                    color: 'text.secondary',
                }}
            >
                Content:{' '}
                <Link
                    color="inherit"
                    href="https://creativecommons.org/licenses/by-nc-sa/4.0/"
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License"
                >
                    CC BY-NC-SA 4.0
                </Link>
                {' | '}
                <Link
                    color="inherit"
                    href="https://github.com/Reklund3/personal-site"
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="View source code on GitHub"
                >
                    Source Code
                </Link>
            </Typography>
        </Box>
    );
}

export default Copyright;