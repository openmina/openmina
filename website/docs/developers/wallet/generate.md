---
title: generate
description: Generate a new encrypted key pair
sidebar_position: 3
---

# generate

Generate a new encrypted key pair for use with Mina.

## Basic usage

```bash
mina wallet generate --output /path/to/key
```

## Arguments

**Required:**

- `--output <PATH>` - Path where the encrypted key file will be saved

**Optional:**

- `[PASSWORD]` - Password to encrypt the key. Can be provided as an argument or
  via the `MINA_PRIVKEY_PASS` environment variable (recommended for security)

## Example

```bash
# Generate new key with environment variable for password
export MINA_PRIVKEY_PASS="my-secret-password"
mina wallet generate --output ./keys/my-new-wallet
```

This command generates a new random keypair, encrypts the private key with the
provided password, and saves it to the specified path. It also creates a `.pub`
file containing the public key.
