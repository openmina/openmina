---
title: address
description: Get the public address from an encrypted key file
sidebar_position: 1
---

# address

Get the public address from an encrypted key file.

## Basic usage

```bash
mina wallet address --from /path/to/encrypted/key
```

## Arguments

**Required:**

- `--from <PATH>` - Path to encrypted key file

**Optional:**

- `[PASSWORD]` - Password to decrypt the key. Can be provided as an argument or
  via the `MINA_PRIVKEY_PASS` environment variable (recommended for security)

## Example

```bash
# Get address from encrypted key
mina wallet address --from ./keys/my-wallet

# Using environment variable for password
export MINA_PRIVKEY_PASS="my-secret-password"
mina wallet address --from ./keys/my-wallet
```

This command simply decrypts the key file and displays the associated public
address. It does not require a connection to a node.
