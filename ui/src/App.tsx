import { Container } from '@mui/material';
import React, { lazy, Suspense, useEffect, useState } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { HelmetProvider } from 'react-helmet-async';
import CircularProgress from '@mui/material/CircularProgress';
import Box from '@mui/material/Box';

// Lazy load components with prefetching
const Summary = lazy(() => {
    const module = import('./components/pages/Summary');
    return module;
});
const Skills = lazy(() => import('./components/pages/Skills'));
const Experience = lazy(() => import('./components/pages/Experience'));
const Education = lazy(() => import('./components/pages/Education'));
const Portfolio = lazy(() => import('./components/pages/Portfolio'));

// Prefetch components
const prefetchComponents = () => {
    // Prefetch all components in the background
    const prefetchPromises = [
        import('./components/pages/Summary'),
        import('./components/pages/Skills'),
        import('./components/pages/Experience'),
        import('./components/pages/Education'),
        import('./components/pages/Portfolio')
    ];

    Promise.all(prefetchPromises).then(() => {
        console.log('All components prefetched');
    });
};

// Loading component
const LoadingComponent = () => (
    <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '50vh' }}>
        <CircularProgress />
    </Box>
);

export default function App() {
    // const location = useLocation();
    const [hasPrefetched, setHasPrefetched] = useState(false);

    // Prefetch components on initial load
    useEffect(() => {
        if (!hasPrefetched) {
            prefetchComponents();
            setHasPrefetched(true);
        }
    }, [hasPrefetched]);

    // Log navigation for debugging
    // useEffect(() => {
    //     console.log('Current route:', location.pathname);
    // }, [location]);

    return (
        <HelmetProvider>
            <Container maxWidth="lg">
                <Suspense fallback={<LoadingComponent />}>
                    <Routes>
                        <Route path="/" element={<Navigate to="/summary" replace />} />
                        <Route path="/summary" element={<Summary />} />
                        <Route path="/skills" element={<Skills />} />
                        <Route path="/experience" element={<Experience />} />
                        <Route path="/education" element={<Education />} />
                        <Route path="/portfolio" element={<Portfolio />} />
                        {/* Redirect old routes to new Portfolio page */}
                        <Route path="/open-source" element={<Navigate to="/portfolio" replace />} />
                        <Route path="/projects" element={<Navigate to="/portfolio" replace />} />
                        <Route path="*" element={<Navigate to="/summary" replace />} />
                    </Routes>
                </Suspense>
            </Container>
        </HelmetProvider>
    );
}
