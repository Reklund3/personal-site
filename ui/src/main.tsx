import React from 'react';
import * as ReactDOM from 'react-dom/client';
import { ThemeProvider } from '@mui/material/styles';
import { CssBaseline } from '@mui/material';
import Box from '@mui/material/Box';
import theme from './theme';
import App from './App';
import ReactiveAppBar from './components/app-bar/ResponsiveAppBar';
import AppFooter from "./components/footer/AppFooter.tsx";
import ContactDialog from './components/ContactDialog';
import {
    createBrowserRouter,
    RouterProvider
} from 'react-router-dom';
import { AppBarHeightProvider, useAppBarHeight } from './context/AppBarHeightContext';
import { ContactDialogProvider, useContactDialog } from './context/ContactDialogContext';
import { SectionNavHeightProvider } from './context/SectionNavHeightContext';

function AppLayoutContent() {
    const { appBarHeight } = useAppBarHeight();
    const { open: contactDialogOpen, closeDialog } = useContactDialog();

    return (
        <Box sx={{
            display: "flex",
            flexDirection: "column",
            minHeight: "100vh", // Ensure content fills at least the full viewport height
        }}>
            <ReactiveAppBar />
            <Box sx={{
                flex: 1,
                marginTop: `${appBarHeight}px`, // Dynamically apply the margin
            }}>
                <App />
            </Box>
            <ContactDialog dialogOpen={contactDialogOpen} onClose={closeDialog} />
            <AppFooter/>
        </Box>
    );
}

function AppLayout() {
    return (
        <AppBarHeightProvider>
            <SectionNavHeightProvider>
                <ContactDialogProvider>
                    <AppLayoutContent />
                </ContactDialogProvider>
            </SectionNavHeightProvider>
        </AppBarHeightProvider>
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
