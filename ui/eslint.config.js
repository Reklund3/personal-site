// Flat config (ESLint 9). `ui/package.json` is `"type": "module"`, so this is ESM.
//
// Added after a review found the `main` landmark missing from the one-pager — the
// kind of defect `tsc` cannot see. jsx-a11y is the point of this config; the rest
// is the baseline that makes it usable.
//
// ESLint is pinned to ^9 deliberately: eslint-plugin-jsx-a11y@6 declares a peer
// ceiling of ^9, and @eslint/js@10 is what drags the resolution to 10 and breaks
// the install. Both must move together when jsx-a11y gains v10 support.
import js from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';
import jsxA11y from 'eslint-plugin-jsx-a11y';

export default tseslint.config(
  { ignores: ['dist/**', 'node_modules/**'] },

  js.configs.recommended,
  // Type-unaware rules only. The type-checked presets need a program per lint run,
  // which is slow enough to discourage running this at all; tsc already covers
  // types in `npm run build`.
  ...tseslint.configs.recommended,
  // Both plugins still ship legacy eslintrc-shaped configs under their top-level
  // `configs` key, where `plugins` is an array of strings and flat config rejects
  // it. The flat-shaped variants live one level deeper, under different names in
  // each plugin — `flatConfigs` here, `configs.flat` there.
  jsxA11y.flatConfigs.recommended,
  reactHooks.configs.flat['recommended-latest'],

  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      globals: globals.browser,
    },
  },

  // Build tooling runs in Node, not the browser.
  {
    files: ['*.config.{js,ts}', 'vite.config.ts'],
    languageOptions: { globals: globals.node },
  },
);
