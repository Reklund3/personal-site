import React from 'react';
import { Box, Typography } from '@mui/material';
import { Link } from 'react-router-dom';

export default function NotFound() {
    return (
        <Box
            component="main"
            // Same skip-link target as the one-pager, so the link in the layout
            // resolves on this route too.
            id="main-content"
            tabIndex={-1}
            sx={{
                outline: 'none',
                my: 4,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                textAlign: 'center',
                minHeight: '50vh',
                gap: 2,
            }}
        >
            <Typography variant="h4" component="h1">
                404 - Page not found
            </Typography>
            <Typography variant="body1" sx={{
                color: "text.secondary"
            }}>
                The page you're looking for doesn't exist.
            </Typography>
            <Link to="/">Back to Home</Link>
        </Box>
    );
}
