import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import Box from '@mui/material/Box';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import { useTheme } from '@mui/material/styles';
import useScrollTrigger from '@mui/material/useScrollTrigger';
import { useSectionNavHeight } from '../../context/SectionNavHeightContext';
import { CONTENT } from '../../content';

/**
 * Map from section ID to its router path.
 * Note: there is no /about route — about maps to '/'.
 */
const SECTION_PATHS = {
  about: '/',
  skills: '/skills',
  experience: '/experience',
  education: '/education',
  portfolio: '/portfolio',
} as const;

export default function SectionNav() {
  const { setNavHeight } = useSectionNavHeight();
  const navigate = useNavigate();
  const location = useLocation();
  const theme = useTheme();
  const navRef = useRef<HTMLDivElement>(null);
  const programmaticScrollRef = useRef<string | null>(null);

  // Scroll-spy active section state
  const [activeSection, setActiveSection] = useState<string | false>('about');

  // Track scroll-triggered background state
  const scrolled = useScrollTrigger({ threshold: 16, disableHysteresis: true });

  /**
   * Measure nav height and publish to context
   */
  useEffect(() => {
    if (!navRef.current) return;

    const resizeObserver = new ResizeObserver(() => {
      if (navRef.current) {
        const h = navRef.current.offsetHeight;
        setNavHeight(h);
      }
    });

    resizeObserver.observe(navRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, [setNavHeight]);

  /**
   * Scroll-spy via IntersectionObserver
   */
  useEffect(() => {
    // Identify the sections to observe
    const sectionIds = ['about', 'skills', 'experience', 'education', 'portfolio'];
    const elements = sectionIds
      .map((id) => ({ id, el: document.getElementById(id) }))
      .filter(({ el }) => el !== null);

    if (elements.length === 0) return;

    // Callback to update active section on intersection changes
    const handleIntersection = (entries: IntersectionObserverEntry[]) => {
      // Filter to only intersecting entries
      const intersecting = entries.filter((entry) => entry.isIntersecting);

      if (intersecting.length === 0) {
        // No intersecting sections — do not change activeSection
        return;
      }

      // Tie-break: pick the one with smallest positive boundingClientRect.top (topmost)
      const topmost = intersecting.reduce((best, entry) => {
        const bestTop = best.target.getBoundingClientRect().top;
        const entryTop = entry.target.getBoundingClientRect().top;
        return entryTop < bestTop ? entry : best;
      });

      const id = (topmost.target as HTMLElement).id;
      setActiveSection(id);
    };

    // Use a top-biased rootMargin so "active" means "near the top of the viewport"
    const observer = new IntersectionObserver(handleIntersection, {
      rootMargin: '-20% 0px -60% 0px',
    });

    elements.forEach(({ el }) => {
      if (el) observer.observe(el);
    });

    return () => {
      observer.disconnect();
    };
  }, []);

  /**
   * Initial scroll on direct deep links
   */
  useEffect(() => {
    // Resolve pathname to section id
    let targetId: string | null = null;

    if (location.pathname === '/') {
      // No scroll needed — already at top
      return;
    } else if (location.pathname === '/skills') {
      targetId = 'skills';
    } else if (location.pathname === '/experience') {
      targetId = 'experience';
    } else if (location.pathname === '/education') {
      targetId = 'education';
    } else if (location.pathname === '/portfolio') {
      targetId = 'portfolio';
    } else if (location.pathname === '/open-source' || location.pathname === '/projects') {
      // Legacy paths map to portfolio
      targetId = 'portfolio';
    }

    if (!targetId) return;

    // If this pathname change was triggered by a tab click, skip the auto-scroll
    // so it doesn't abort the in-flight smooth scroll.
    if (programmaticScrollRef.current === targetId) {
      programmaticScrollRef.current = null;
      return;
    }

    // Wait for layout to settle (after fonts/images), then scroll
    requestAnimationFrame(() => {
      const element = document.getElementById(targetId!);
      if (element) {
        element.scrollIntoView({ behavior: 'auto', block: 'start' });
      }
    });
  }, [location.pathname]);

  /**
   * Handle tab click: set active, scroll, update URL
   */
  const handleTabChange = useCallback(
    (event: React.SyntheticEvent, newValue: string) => {
      event.preventDefault();

      // Set active section immediately
      setActiveSection(newValue);

      // Get the target element
      const element = document.getElementById(newValue);
      if (!element) return;

      // Flag that this scroll was initiated programmatically by tab click
      programmaticScrollRef.current = newValue;

      // Determine smooth vs instant based on reduced motion
      const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      const behavior = prefersReducedMotion ? 'auto' : 'smooth';

      // Scroll to section (scrollMarginTop handles the offset)
      element.scrollIntoView({ behavior, block: 'start' });

      // Update URL with router-aware navigation
      const path = SECTION_PATHS[newValue as keyof typeof SECTION_PATHS];
      if (path) {
        navigate(path, { replace: true });
      }
    },
    [navigate]
  );

  return (
    <Box
      ref={navRef}
      sx={{
        position: 'sticky',
        top: 0,
        zIndex: theme.zIndex.appBar - 1,
        bgcolor: scrolled ? 'rgba(15, 15, 15, 0.97)' : 'rgba(18, 18, 18, 0.8)',
        borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
        boxShadow: scrolled ? '0 6px 20px rgba(0, 0, 0, 0.45)' : 'none',
        transition: 'all 0.25s ease',
      }}
    >
      <Tabs
        value={activeSection}
        onChange={handleTabChange}
        role="navigation"
        aria-label="Section navigation"
        variant="scrollable"
        scrollButtons={false}
        textColor="primary"
        indicatorColor="primary"
        sx={{
          '& .MuiTabs-indicator': {
            height: '2px',
          },
          '& .MuiTab-root': {
            fontSize: '12px',
            fontWeight: 600,
            textTransform: 'none',
            padding: '8px 12px 6px',
            color: 'rgba(255, 255, 255, 0.65)',
            minHeight: 'auto',
            '&.Mui-selected': {
              fontWeight: 700,
              color: 'primary.main',
            },
          },
          // Center tabs on desktop only
          '& .MuiTabs-flexContainer': {
            justifyContent: { xs: 'flex-start', md: 'center' },
          },
        }}
      >
        {CONTENT.navItems.map(({ id, label }) => (
          <Tab
            key={id}
            value={id}
            label={label}
          />
        ))}
      </Tabs>
    </Box>
  );
}
