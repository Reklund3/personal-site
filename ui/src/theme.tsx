import { createTheme } from '@mui/material/styles';

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
                primary: {
                    main: '#90caf9',
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
            fontSize: 11.5,
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
                    borderRadius: 20,
                    textTransform: 'none',
                },
            },
        },
    },
});

export default theme;