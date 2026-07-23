import React from 'react';
import { Box, Paper, Typography } from '@mui/material';
import { SEOMetaTags } from '../../utils/seo';

export default function Education() {
    return (
        <>
            <SEOMetaTags path="/education" />
            <Box sx={{ my: 4 }}>
            <Paper elevation={5} square={false} sx={{ p: 2 }}>
                <Typography variant="body1" sx={{mb: 2}}>
                    Texas State University, San Marcos, Tx — Masters in Accounting Information Systems
                </Typography>
                <Typography variant="body1" sx={{mb: 2}}>
                    Texas State University, San Marcos, Tx — Bachelor's in Accounting
                </Typography>
            </Paper>
        </Box>
        </>
    );
}