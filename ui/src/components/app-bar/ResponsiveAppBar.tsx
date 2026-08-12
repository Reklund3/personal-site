import React, {useRef, useEffect, useState, useCallback, memo} from 'react';
import AppBar from '@mui/material/AppBar';
import Avatar from '@mui/material/Avatar';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import IconButton from '@mui/material/IconButton';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import Toolbar from '@mui/material/Toolbar';
import Typography from '@mui/material/Typography';
import AccountCircle from '@mui/icons-material/AccountCircle';
import CodeIcon from '@mui/icons-material/Code';
import Download from '@mui/icons-material/Download';
import GitHub from '@mui/icons-material/GitHub';
import LinkedIn from '@mui/icons-material/LinkedIn';
import MenuIcon from '@mui/icons-material/Menu';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAppBarHeight } from '../../context/AppBarHeightContext';
import { useContactDialog } from '../../context/ContactDialogContext';

function ResponsiveAppBarComponent() {
    const appBarRef = useRef<HTMLDivElement>(null);
    const { setAppBarHeight, appBarHeight } = useAppBarHeight();
    const { openDialog } = useContactDialog();

    // Use ResizeObserver to track AppBar height changes
    useEffect(() => {
        if (!appBarRef.current) return;

        const resizeObserver = new ResizeObserver(() => {
            if (appBarRef.current) {
                const h = appBarRef.current.offsetHeight;
                // Guard against redundant writes
                if (h !== appBarHeight) {
                    setAppBarHeight(h);
                }
            }
        });

        resizeObserver.observe(appBarRef.current);

        return () => {
            resizeObserver.disconnect();
        };
    }, [appBarHeight, setAppBarHeight]);

    const navigate = useNavigate();
    const location = useLocation();
    const [anchorElNav, setAnchorElNav] = useState<null | HTMLElement>(null);

    const handleOpenNavMenu = useCallback((event: React.MouseEvent<HTMLElement>) => {
        setAnchorElNav(event.currentTarget);
    }, []);

    const handleCloseNavMenu = useCallback(() => {
        setAnchorElNav(null);
    }, []);

    const handleLogoClick = useCallback((event: React.MouseEvent<HTMLElement>) => {
        event.preventDefault();
        if (location.pathname === "/") {
            // On home page, scroll to top
            const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
            window.scrollTo({ top: 0, behavior: prefersReducedMotion ? 'auto' : 'smooth' });
        } else {
            // On other pages, navigate to home
            navigate("/");
        }
    }, [location.pathname, navigate]);

    return (
        <AppBar position="fixed" ref={appBarRef}>
            <Container maxWidth="xl">
                {/* Desktop: Two-row layout with avatar spanning both rows */}
                <Box sx={{ display: { xs: 'none', md: 'flex' }, alignItems: 'stretch' }}>
                    {/* Left: Avatar spanning both rows */}
                    <Box
                        sx={{
                            display: 'flex',
                            alignItems: 'center',
                            pr: 2,
                            borderRight: 1,
                            borderColor: 'divider',
                        }}
                    >
                        <Avatar
                            src="/headshot"
                            alt="Robert Eklund"
                            slotProps={{
                                img: {
                                    onError: (e: React.SyntheticEvent<HTMLImageElement>) => {
                                        e.currentTarget.style.display = 'none';
                                    }
                                }
                            }}
                            sx={{
                                width: 64,
                                height: 64,
                                bgcolor: 'primary.main',
                            }}
                        >
                            <AccountCircle sx={{ fontSize: '3rem' }} />
                        </Avatar>
                    </Box>

                    {/* Right: Two rows of content */}
                    <Box sx={{ display: 'flex', flexDirection: 'column', flexGrow: 1, pl: 2 }}>
                        {/* Row 1: Name + Title + Social Links */}
                        <Box
                            sx={{
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'space-between',
                                borderBottom: 1,
                                borderColor: 'divider',
                                py: 1,
                            }}
                        >
                            {/* Name + Title */}
                            <Box>
                                <Typography
                                    variant="h6"
                                    component="a"
                                    href="/"
                                    onClick={handleLogoClick}
                                    sx={{
                                        fontWeight: 700,
                                        color: 'inherit',
                                        textDecoration: 'none',
                                        display: 'block',
                                        lineHeight: 1.2,
                                    }}
                                >
                                    Robert Eklund
                                </Typography>
                                <Typography
                                    variant="caption"
                                    sx={{
                                        color: 'text.secondary',
                                        display: 'block',
                                        lineHeight: 1,
                                    }}
                                >
                                    Software Engineer
                                </Typography>
                            </Box>

                            {/* Social Links + Resume Button */}
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                                <IconButton
                                    color="inherit"
                                    href="https://github.com/Reklund3"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    size="small"
                                >
                                    <GitHub />
                                </IconButton>
                                <IconButton
                                    color="inherit"
                                    href="https://www.linkedin.com/in/robert-eklund-64302976/"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    size="small"
                                >
                                    <LinkedIn />
                                </IconButton>
                                <Button
                                    startIcon={<Download />}
                                    href="/resume"
                                    size="small"
                                    sx={{ ml: 1 }}
                                >
                                    Resume
                                </Button>
                            </Box>
                        </Box>

                        {/* Row 2: Contact CTA (section nav moved to SectionNav in Phase 5) */}
                        <Box
                            sx={{
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'flex-end',
                                py: 1,
                            }}
                        >
                            {/* Contact Button */}
                            <Button
                                onClick={openDialog}
                                variant="contained"
                            >
                                Contact Now
                            </Button>
                        </Box>
                    </Box>
                </Box>

                {/* Mobile: Single row navigation */}
                <Toolbar disableGutters sx={{ display: { xs: 'flex', md: 'none' } }}>
                    <IconButton
                        size="large"
                        aria-label="Robert Eklund Resume Menu"
                        aria-controls="menu-appbar"
                        aria-haspopup="true"
                        onClick={handleOpenNavMenu}
                        color="inherit"
                    >
                        <MenuIcon />
                    </IconButton>
                    <Menu
                        id="menu-appbar"
                        anchorEl={anchorElNav}
                        anchorOrigin={{
                            vertical: 'bottom',
                            horizontal: 'left',
                        }}
                        keepMounted
                        transformOrigin={{
                            vertical: 'top',
                            horizontal: 'left',
                        }}
                        open={Boolean(anchorElNav)}
                        onClose={handleCloseNavMenu}
                        sx={{ display: { xs: 'block', md: 'none' } }}
                    >
                        <MenuItem
                            component="a"
                            href="/resume"
                            onClick={handleCloseNavMenu}
                        >
                            <Download sx={{ mr: 1 }} />
                            <Typography sx={{ textAlign: 'center' }}>Resume</Typography>
                        </MenuItem>
                    </Menu>

                    <CodeIcon sx={{ mr: 1 }} />
                    <Typography
                        variant="h6"
                        noWrap
                        href="/"
                        onClick={handleLogoClick}
                        component="a"
                        sx={{
                            mr: 2,
                            flexGrow: 1,
                            fontWeight: 700,
                            color: 'inherit',
                            textDecoration: 'none',
                        }}
                    >
                        Robert Eklund
                    </Typography>

                    <Button
                        onClick={openDialog}
                        variant="contained"
                    >
                        Contact Now
                    </Button>
                </Toolbar>
            </Container>
        </AppBar>
    );
}
// Memoize the component to prevent unnecessary re-renders
const ResponsiveAppBar = memo(ResponsiveAppBarComponent);
export default ResponsiveAppBar;
