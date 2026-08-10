import { Typography } from "@mui/material";
import React from "react";

export default function PageTitle({ children }: { children: React.ReactNode }) {
    return (
        <Typography variant="h4" component="h1" sx={{ mb: 3, fontWeight: 600 }}>
            {children}
        </Typography>
    );
}
