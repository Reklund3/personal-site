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
        <Box>
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
