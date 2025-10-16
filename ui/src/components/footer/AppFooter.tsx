import Copyright from "../Copyright.tsx";
import { AppBar, Box } from "@mui/material";
import React from "react";

function AppFooter() {
    return (
        <AppBar
            component="footer"
            sx={{
                position: 'relative',
                display: 'flex',
                flexDirection: 'row',
                justifyContent: 'center',
                p: 0,
                bottom: 0,
                left: 0,
                right: 0,
            }}
        >
            <Box
                sx={{
                    display: 'flex',
                    flexDirection: 'row',
                    justifyContent: 'space-around',
                    maxWidth: 'xl',
                    width: '100%'
                }}
            >
                <Copyright />
            </Box>
        </AppBar>
    )
}

export default AppFooter;