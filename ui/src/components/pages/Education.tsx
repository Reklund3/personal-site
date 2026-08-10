import React from 'react';
import { Box, Paper, Typography } from '@mui/material';
import { SEOMetaTags } from '../../utils/seo';
import PageTitle from '../PageTitle';

export default function Education() {
    return (
        <>
            <SEOMetaTags path="/education" />
            <Box component="main" sx={{ my: 4 }}>
            <Paper component="article" elevation={5} square={false} sx={{ p: 3 }}>
                <PageTitle>Education</PageTitle>
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