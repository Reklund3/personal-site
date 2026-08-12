import Copyright from "../Copyright";
import { Box } from "@mui/material";
import React from "react";

function AppFooter() {
    return (
        <Box component="footer" sx={{ width: '100%', mt: 'auto' }}>
            <Copyright />
        </Box>
    );
}

export default AppFooter;
