import React from 'react';
import { Box, Typography } from '@mui/material';
import { Link } from 'react-router-dom';

export default function NotFound() {
    return (
        <Box
            sx={{
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
            <Typography variant="body1" color="text.secondary">
                The page you're looking for doesn't exist.
            </Typography>
            <Link to="/">Back to Home</Link>
        </Box>
    );
}
