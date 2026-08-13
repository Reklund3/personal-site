import React from 'react';
import * as ReactDOM from 'react-dom/client';
import { ThemeProvider } from '@mui/material/styles';
import { CssBaseline } from '@mui/material';
import Box from '@mui/material/Box';
import theme from './theme';
import App from './App';
import AppFooter from "./components/footer/AppFooter.tsx";
import ContactDialog from './components/ContactDialog';
import {
    createBrowserRouter,
    RouterProvider
} from 'react-router-dom';
import { ContactDialogProvider, useContactDialog } from './context/ContactDialogContext';
import { SectionNavHeightProvider } from './context/SectionNavHeightContext';

function AppLayoutContent() {
    const { open: contactDialogOpen, closeDialog } = useContactDialog();

    return (
        <Box sx={{
            display: "flex",
            flexDirection: "column",
            minHeight: "100vh", // Ensure content fills at least the full viewport height
        }}>
            {/*
                Skip link. Must be the first focusable element on the page, which is
                why it lives here rather than inside a route component. Off-screen
                until focused, then pinned to the top-left.

                Sighted keyboard users otherwise tab through the whole sticky
                section nav before reaching content; screen reader users could
                already jump via the `main` landmark, but that does not help anyone
                navigating by Tab alone.
            */}
            <Box
                component="a"
                href="#main-content"
                sx={{
                    position: 'absolute',
                    left: '-9999px',
                    top: 0,
                    zIndex: (t) => t.zIndex.tooltip,
                    p: '10px 16px',
                    bgcolor: 'primary.main',
                    color: 'primary.contrastText',
                    fontSize: 13,
                    fontWeight: 700,
                    borderRadius: '0 0 6px 0',
                    textDecoration: 'none',
                    '&:focus': { left: 0 },
                }}
            >
                Skip to main content
            </Box>
            <Box sx={{ flex: 1 }}>
                <App />
            </Box>
            <ContactDialog dialogOpen={contactDialogOpen} onClose={closeDialog} />
            <AppFooter/>
        </Box>
    );
}

function AppLayout() {
    return (
        <SectionNavHeightProvider>
            <ContactDialogProvider>
                <AppLayoutContent />
            </ContactDialogProvider>
        </SectionNavHeightProvider>
    );
}

// History-based router: real paths (/skills), no hash. The server enumerates
// these same paths in src/startup.rs so deep links and hard refreshes work.
const router = createBrowserRouter(
    [
        {
            path: "*",
            element: <AppLayout />
        }
    ]
);

ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
        <ThemeProvider theme={theme}>
            <CssBaseline />
            <RouterProvider router={router} />
        </ThemeProvider>
    </React.StrictMode>,
);
