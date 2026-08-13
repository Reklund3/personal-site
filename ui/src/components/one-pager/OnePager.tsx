import React from 'react';
import { useLocation } from 'react-router-dom';
import Box from '@mui/material/Box';
import Masthead from '../sections/Masthead';
import SectionNav from '../nav/SectionNav';
import AboutSection from '../sections/AboutSection';
import SkillsSection from '../sections/SkillsSection';
import ExperienceSection from '../sections/ExperienceSection';
import EducationSection from '../sections/EducationSection';
import PortfolioSection from '../sections/PortfolioSection';
import { SEOMetaTags } from '../../utils/seo';

/**
 * OnePager component - the main scrolling one-pager layout.
 * Contains: Masthead → About → Skills → Experience → Education → Portfolio
 * (Footer is rendered by AppLayout in main.tsx, not here.)
 */
export default function OnePager() {
    const location = useLocation();

    return (
        // `main` landmark. Every page component on the pre-one-pager site carried
        // one; the rework dropped it everywhere except NotFound, leaving the entire
        // page outside any landmark and removing "skip to main content" for screen
        // readers. It sits on the outer Box rather than wrapping only the sections
        // so the masthead — which owns the h1 — stays inside it. The tradeoff is
        // that SectionNav's navigation landmark nests inside this one, which is
        // valid but not ideal; hoisting the nav out would put the h1 outside main.
        // `tabIndex={-1}` so the skip link can actually move focus here; without it
        // the browser scrolls to the anchor but focus stays on the link, and the
        // next Tab returns to the nav the user just skipped.
        <Box component="main" id="main-content" tabIndex={-1} sx={{ outline: 'none' }}>
            {/* Keeps document.title in sync on client-side navigation; renders no DOM */}
            <SEOMetaTags path={location.pathname} />
            <Masthead />
            <SectionNav />
            <AboutSection />
            <SkillsSection />
            <ExperienceSection />
            <EducationSection />
            <PortfolioSection />
        </Box>
    );
}
