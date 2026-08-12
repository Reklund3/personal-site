import { createTheme } from '@mui/material/styles';
import { blue } from '@mui/material/colors';

// Module augmentation for surface token
declare module '@mui/material/styles' {
    interface Palette {
        surface: { alt: string };
    }
    interface PaletteOptions {
        surface?: { alt?: string };
    }
}

// Module augmentation for eyebrow typography variant
declare module '@mui/material/styles' {
    interface TypographyVariants {
        eyebrow: React.CSSProperties;
    }
    interface TypographyVariantsOptions {
        eyebrow?: React.CSSProperties;
    }
}

declare module '@mui/material/Typography' {
    interface TypographyPropsVariantOverrides {
        eyebrow: true;
    }
}

// A custom theme for this app
const theme = createTheme({
    cssVariables: true,
    colorSchemes: {
        dark: {
            palette: {
                // `main` is REQUIRED here: SimplePaletteColorOptions declares `main: string`
                // (not optional), so contrastText cannot be overridden on its own. Sourcing it
                // from blue[200] rather than a literal keeps it visibly identical to MUI's dark
                // default instead of forking the accent into a second source of truth.
                primary: {
                    main: blue[200],
                    contrastText: '#062341',
                },
                surface: {
                    alt: '#171717',
                },
            },
        },
    },
    defaultColorScheme: 'dark',
    typography: {
        eyebrow: {
            // String, not a bare number: with cssVariables enabled MUI generates a
            // `font` shorthand var per typography key, and a unitless value emits the
            // invalid `--mui-font-eyebrow: 700 11.5/1`.
            fontSize: '11.5px',
            fontWeight: 700,
            letterSpacing: '1.5px',
            textTransform: 'uppercase',
            lineHeight: 1,
        },
    },
    components: {
        MuiTypography: {
            defaultProps: {
                variantMapping: {
                    eyebrow: 'h2',
                },
            },
        },
        MuiCard: {
            styleOverrides: {
                root: {
                    borderRadius: 10,
                },
            },
        },
        MuiChip: {
            styleOverrides: {
                root: {
                    borderRadius: 12,
                    fontWeight: 500,
                },
            },
        },
        MuiButton: {
            styleOverrides: {
                root: {
                    // borderRadius is deliberately NOT set here. 20px is the masthead
                    // pill radius, not a site-wide value — globally it would also
                    // restyle every ContactDialog button. The two pills carry it in
                    // their own sx instead.
                    textTransform: 'none',
                },
            },
        },
    },
});

export default theme;