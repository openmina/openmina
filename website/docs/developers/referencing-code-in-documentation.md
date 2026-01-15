---
sidebar_position: 10
title: Referencing code in documentation
description: Best practices for writing and maintaining documentation
slug: /developers/referencing-code-in-documentation
---

# Referencing code in documentation

This guide explains how to write and maintain documentation for the Mina Rust
project, including how to reference code from the codebase. Referencing code
from codebases can be useful to check compatibility between implementations. For
instance, we can have pages where the Rust code is compared to the OCaml code to
discuss the differences or similarities.

To keep documentation synchronized with the actual codebase, we use the
[`docusaurus-theme-github-codeblock`](https://github.com/christian-bromann/docusaurus-theme-github-codeblock)
plugin that automatically fetches code from GitHub.

### How to add code references

Use this pattern to reference code snippets:

````
<!-- CODE_REFERENCE: path/to/file.rs#LStartLine-LEndLine -->

```rust reference title="path/to/file.rs"
https://github.com/o1-labs/mina-rust/blob/develop/path/to/file.rs#LStartLine-LEndLine
```
````

### Example

Here's a real example from the zkApps documentation:

<!-- CODE_REFERENCE: crates/ledger/src/scan_state/transaction_logic/valid.rs#L80-L83-->

```rust reference title="crates/ledger/src/scan_state/transaction_logic/valid.rs"
https://github.com/o1-labs/mina-rust/blob/develop/crates/ledger/src/scan_state/transaction_logic/valid.rs#L80-L83
```

### Components explained

1. **CODE_REFERENCE comment**: Acts as the source of truth for verification

   ```markdown
   <!-- CODE_REFERENCE: path/to/file.rs#LStartLine-LEndLine -->
   ```

   - Must match the GitHub URL line range exactly
   - Used by CI to verify references are valid
   - Path is relative to repository root

2. **Code block with reference**: Displays the actual code

   ```markdown
   (triple backticks)rust reference title="path/to/file.rs"
   https://github.com/o1-labs/mina-rust/blob/develop/path/to/file.rs#LStartLine-LEndLine
   (triple backticks)
   ```

   - Language: Use appropriate language identifier (`rust`, `toml`, `bash`,
     etc.)
   - `reference` keyword: Tells the plugin to fetch code from GitHub
   - `title`: Optional, shows the file path above the code block
   - URL: Full GitHub URL with line range (`#L10-L20`)

### Verification

A verification script runs in CI to ensure all code references are valid:

```bash
bash .github/scripts/verify-code-references.sh
```

The script checks:

- ✓ Referenced files exist
- ✓ Line ranges are valid
- ✓ GitHub URLs match CODE_REFERENCE comments
- ✓ Code blocks have corresponding references
- ✓ Local code matches what's deployed on GitHub

### When code changes

If code is added or removed and line numbers shift:

1. The verification script will detect the mismatch in CI
2. Update the `CODE_REFERENCE` comment with new line numbers
3. Update the GitHub URL with matching line numbers
4. The plugin will automatically fetch the updated code

### Important: workflow for adding code references

<!-- prettier-ignore-start -->

:::caution

Code references must always point to code that exists on the `develop` branch.
The verification script compares local code with GitHub's `develop` branch.

:::

<!-- prettier-ignore-end -->

**Recommended workflow:**

1. **First PR**: Implement and merge your code changes to `develop`
2. **Second PR**: Add documentation with code references

This ensures:

- Code references are always valid in CI
- Documentation doesn't break due to uncommitted changes
- The plugin can fetch code from GitHub successfully

**Why separate PRs?**

- The verification script compares local code with GitHub's `develop` branch
- If code isn't merged yet, the script will fail with a "code mismatch" error
- This prevents documentation from referencing code that doesn't exist on
  `develop`
