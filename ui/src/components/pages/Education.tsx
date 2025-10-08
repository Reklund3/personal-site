import React from 'react';
import { Box, Paper, Typography } from '@mui/material';
import { SEOMetaTags } from '../../utils/seo';

export default function Education() {
    return (
        <>
            <SEOMetaTags
                title="Education"
                description="Masters in Accounting Information Systems from Texas State University, MBA from St. Edward's University, and full-stack development training from Austin Coding Academy."
                keywords="Texas State University, St. Edward's University, Austin Coding Academy, Masters in Accounting, MBA, Full-Stack Development"
            />
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