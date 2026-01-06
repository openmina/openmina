# mina-rust-shared

Shared Angular library for the Mina Rust frontend application, containing
reusable components, directives, pipes, services, and utilities.

## Overview

This library provides common functionality used across the Mina Rust frontend:

- **Components**: Reusable UI components (tables, graphs, tooltips, etc.)
- **Directives**: Common directives (click-outside, copy-to-clipboard, tooltips)
- **Pipes**: Data transformation pipes (date formatting, size conversion, etc.)
- **Services**: Shared services (theme switching, error handling, tooltips)
- **Base Classes**: Abstract classes for common patterns
- **Helpers**: Utility functions for arrays, dates, routing, etc.
- **Types**: Shared TypeScript types and interfaces

## Location

This library is located in the vendor directory as a local dependency and is not
published to npm. It is used exclusively by the Mina Rust frontend application.

## Development

Since this is a local vendor library, changes are immediately reflected in the
main application without requiring a build step.

## Styles

Component styles use SCSS and import from the shared styles library:

```scss
@use 'mina-rust' as *;
```

The main styles are bundled and managed by the `mina-rust-styles` vendor library.
