import React from 'react';
import { Routes, Route } from 'react-router-dom';
import OnePager from './components/one-pager/OnePager';
import NotFound from './components/pages/NotFound';

export default function App() {
    return (
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
    );
}
