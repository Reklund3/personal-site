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
