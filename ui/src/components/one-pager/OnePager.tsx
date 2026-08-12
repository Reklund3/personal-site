import React from 'react';
import Box from '@mui/material/Box';
import Masthead from '../sections/Masthead';
import AboutSection from '../sections/AboutSection';
import SkillsSection from '../sections/SkillsSection';
import ExperienceSection from '../sections/ExperienceSection';
import EducationSection from '../sections/EducationSection';
import PortfolioSection from '../sections/PortfolioSection';
import Copyright from '../Copyright';

/**
 * OnePager component - the main scrolling one-pager layout.
 * Contains: Masthead → About → Skills → Experience → Education → Portfolio → Footer
 */
export default function OnePager() {
    return (
        <Box>
            <Masthead />
            <AboutSection />
            <SkillsSection />
            <ExperienceSection />
            <EducationSection />
            <PortfolioSection />
            <Copyright />
        </Box>
    );
}
