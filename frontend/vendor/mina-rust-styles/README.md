# mina-rust-styles

SCSS styles library for the Mina Rust frontend application, providing a unified
design system and shared style utilities.

## Overview

This library contains all the shared SCSS styles used across the Mina Rust
frontend:

- **Components**: Button, input, table, and tooltip styles
- **Material**: Material UI component overrides (accordion, icons, popups)
- **Utilities**: SCSS utility classes for common styling patterns
  - Backgrounds, dimensions, flex layouts
  - Font styles, margins, paddings
  - Text colors and utilities
  - Scrollbar customization
  - Design system variables

## Location

This library is located in the vendor directory as a local dependency and is not
published to npm. It is used exclusively by the Mina Rust frontend application.

## Building Styles

The styles are bundled using `scss-bundle` to create a single consolidated SCSS
file:

```bash
# From the frontend directory
npm run vendor:styles

# Or from the mina-rust-styles directory
npm run update:styles
```

This will:

1. Bundle all SCSS files into a single `mina-rust.scss` file
2. Copy the bundled file to:

- `frontend/src/assets/styles/mina-rust.scss`
- `frontend/vendor/mina-rust-shared/src/assets/styles/mina-rust.scss`

## Development

The entry point for the styles is `src/lib/styles/entry.scss`, which imports all
component and utility styles.

When making changes to styles:

1. Edit the source SCSS files in `src/lib/styles/`
2. Run `npm run update:styles` to rebuild the bundled file
3. The changes will be reflected in the frontend application
