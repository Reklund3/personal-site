import React, {useRef, useLayoutEffect, useState, useCallback, memo} from 'react';
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
import {menuItemsTitles} from "../constants/constants.ts";
import ContactDialog from "../ContactDialog.tsx";
import { useNavigate, useLocation } from 'react-router-dom';

interface ResponsiveAppBarProps {
    onHeightMeasured: (height: number) => void;
}

function ResponsiveAppBarComponent({ onHeightMeasured }: ResponsiveAppBarProps) {
    const appBarRef = useRef<HTMLDivElement>(null);
    useLayoutEffect(() => {
        if (appBarRef.current) {
            // Measure the height of the AppBar
            onHeightMeasured(appBarRef.current.offsetHeight);
        }
    }, [onHeightMeasured]);

    const navigate = useNavigate();
    const location = useLocation();
    const [contactDialogOpen, setContactDialogOpen] = useState(false);
    const [anchorElNav, setAnchorElNav] = useState<null | HTMLElement>(null);

    const handleOpenNavMenu = useCallback((event: React.MouseEvent<HTMLElement>) => {
        setAnchorElNav(event.currentTarget);
    }, []);

    const handleCloseNavMenu = useCallback(() => {
        setAnchorElNav(null);
    }, []);

    const handleMenuItemClick = useCallback((menuItem: string) => {
        handleCloseNavMenu();
        const path = menuItem === 'Open Source' ? '/open-source' : `/${menuItem.toLowerCase()}`;
        // Only navigate if we're not already on this path
        if (location.pathname !== path) {
            navigate(path);
        }
    }, [handleCloseNavMenu, location.pathname, navigate]);

    const handleContactClick = useCallback(() => {
        setContactDialogOpen(true);
    }, []);

    const handleLogoClick = useCallback((event: React.MouseEvent<HTMLElement>) => {
        // Prevent default browser navigation
        event.preventDefault();
        // Only navigate if we're not already on the summary page
        if (location.pathname !== "/summary" && location.pathname !== "/") {
            navigate("/summary");
        }
    }, [location.pathname, navigate]);

    const getSelectedOption = useCallback(() => {
        const path = location.pathname;
        if (path === '/open-source') return 'Open Source';
        if (path === '/') return 'Summary';
        return path.substring(1).charAt(0).toUpperCase() + path.substring(2);
    }, [location.pathname]);

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
                                    href="/summary"
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

                        {/* Row 2: Navigation Links + Contact */}
                        <Box
                            sx={{
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'space-between',
                                py: 1,
                            }}
                        >
                            {/* Desktop Navigation Links */}
                            <Box sx={{ display: 'flex', gap: 1 }}>
                                {menuItemsTitles.map((page) => (
                                    <Button
                                        key={page}
                                        disableRipple={true}
                                        onClick={(() => handleMenuItemClick(page))}
                                        sx={{
                                            color: getSelectedOption() === page ? 'primary' : 'white',
                                            textDecoration: getSelectedOption() === page ? 'underline' : 'none',
                                            fontWeight: getSelectedOption() === page ? 'bold' : 'normal',
                                            whiteSpace: 'nowrap',
                                            px: 2,
                                            transition: 'all 0.4s ease-out',
                                            fontSize: '1rem',
                                            "&:hover": {
                                                backgroundColor: "inherit",
                                            }
                                        }}
                                        size={getSelectedOption() === page ? "medium" : "small"}
                                    >
                                        {page}
                                    </Button>
                                ))}
                            </Box>

                            {/* Contact Button */}
                            <Button
                                onClick={handleContactClick}
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
                        {menuItemsTitles.map((page) => (
                            <MenuItem key={page} onClick={(() => handleMenuItemClick(page))}>
                                <Typography sx={{ textAlign: 'center' }}>{page}</Typography>
                            </MenuItem>
                        ))}
                    </Menu>

                    <CodeIcon sx={{ mr: 1 }} />
                    <Typography
                        variant="h6"
                        noWrap
                        href="/summary"
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
                        onClick={handleContactClick}
                        variant="contained"
                    >
                        Contact Now
                    </Button>
                </Toolbar>

                <ContactDialog dialogOpen={contactDialogOpen} onClose={() => setContactDialogOpen(false)}/>
            </Container>
        </AppBar>
    );
}
// Memoize the component to prevent unnecessary re-renders
const ResponsiveAppBar = memo(ResponsiveAppBarComponent);
export default ResponsiveAppBar;
