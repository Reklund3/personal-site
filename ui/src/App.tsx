import React, { lazy, Suspense } from 'react';
import { Routes, Route } from 'react-router-dom';
import CircularProgress from '@mui/material/CircularProgress';
import Box from '@mui/material/Box';
import OnePager from './components/one-pager/OnePager';

// Lazy load NotFound page
const NotFound = lazy(() => import('./components/pages/NotFound'));

// Loading component
const LoadingComponent = () => (
    <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '50vh' }}>
        <CircularProgress />
    </Box>
);

export default function App() {
    return (
        <Suspense fallback={<LoadingComponent />}>
            <Routes>
                <Route path="/" element={<OnePager />} />
                <Route path="/skills" element={<OnePager />} />
                <Route path="/experience" element={<OnePager />} />
                <Route path="/education" element={<OnePager />} />
                <Route path="/portfolio" element={<OnePager />} />
                {/* legacy paths — land on the Portfolio anchor, no redirect */}
                <Route path="/open-source" element={<OnePager />} />
                <Route path="/projects" element={<OnePager />} />
                <Route path="*" element={<NotFound />} />
            </Routes>
        </Suspense>
    );
}
