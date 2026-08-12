import { Box, Container, Link, Typography } from "@mui/material";
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
                py: '22px',
                textAlign: 'center',
            }}
        >
            <Container maxWidth="lg">
                <Typography
                    sx={{
                        fontSize: 10.5,
                        color: 'rgba(255,255,255,.5)',
                    }}
                >
                    © {currentYear} Robert Eklund · Licensed under{' '}
                    <Link
                        href="https://creativecommons.org/licenses/by-nc-sa/4.0/"
                        target="_blank"
                        rel="noopener noreferrer"
                        color="inherit"
                        underline="hover"
                        sx={{
                            color: 'rgba(255,255,255,.5)',
                            '&:hover': {
                                color: 'rgba(255,255,255,.7)',
                            },
                        }}
                    >
                        CC BY-NC-SA 4.0
                    </Link>
                </Typography>
            </Container>
        </Box>
    );
}

export default Copyright;