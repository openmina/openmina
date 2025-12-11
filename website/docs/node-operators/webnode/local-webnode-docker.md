---
sidebar_position: 1
title: Local WebNode (Docker)
description: How to deploy and launch a webnode using Docker images
---

# Deploying and launching a webnode using Docker

If you intend to just run the webnode instead of actively developing it, it is
substantially easier to get set up using prebuilt Docker images. These images
are automatically built on every versioned release.

## Steps

### 1. Generate a node key

:::info This step should be redundant in a future version.

:::

The current version of the webnode requires you to supply a node key, even if
you don't plan to produce blocks, or you supply your own key archive. This can
be generated once and reused across launches of the webnode container.

```sh
# Using the latest version of the Rust node, generate a keypair.
# The --web-node-secrets flag formats the generated keypair into JSON the webnode can use.
docker run --rm o1labs/mina-rust:latest misc mina-key-pair --web-node-secrets > $HOME/web-node-key-pair.json
```

### 2. Launch the webnode container

You can now simply launch the webnode as a container. Note that once
pre-generating a node keypair is no longer necessary, the `-v` can be removed.

```sh
# Launch the latest version of the frontend in webnode configuration.
# We mount the keypair generated in step 1, and bind port 4200 on the host to 80 (http) in the container
docker run \
  -e MINA_FRONTEND_ENVIRONMENT=webnode \
  -v ~/web-node-key-pair.json:/usr/local/apache2/htdocs/assets/webnode/web-node-secrets.json \
  -p 4200:80 \
  o1labs/mina-rust-frontend:latest
```

### 3. Open your browser

Navigate to [http://localhost:4200](http://localhost:4200) and enjoy using the
webnode! If you used a different port (`-p`) when launching the container, then
update the port accordingly.

## Environment Variable Reference

The Dockerized WebNode can be configured with additional environment variables
to customize behavior.

### `MINA_FRONTEND_ENVIRONMENT`

This is required to launch the frontend in general. To serve a webnode, it must
be set to the value `webnode`.

Example:

```
-e MINA_FRONTEND_ENVIRONMENT=webnode
```

### `MINA_WEBNODE_SEED_URLS`

A comma-separated list of http(s) URLs to fetch initial P2P seeds from. This is
similar to the `--peer-list-url` flag in the native node, but can take multiple
comma-separated values to fetch peer lists from multiple sources. Each peer list
file must contain peer addresses in
[WebRTC-Multiaddrish format](../../developers/webrtc.md#address-format-differences),
separated by a newline.

Example:

```
-e MINA_WEBNODE_SEED_URLS=https://bootnodes.minaprotocol.com/networks/devnet-webrtc.txt
-e MINA_WEBNODE_SEED_URLS=https://example.com/extra-peers.txt,https://bootnodes.minaprotocol.com/networks/devnet-webrtc.txt,
```

### `MINA_WEBNODE_BOOTNODES`

A comma separated list of
[WebRTC-Multiaddrish](../../developers/webrtc.md#address-format-differences)
peer addresses, to attempt to connect to in conjunction to any downloaded from
`MINA_WEBNODE_SEED_URLS`. This is akin to the `--peers` flag in the native node,
but instead of specifying the flag multiple times, each peer is simply
comma-separated

Example:

```
-e MINA_WEBNODE_BOOTNODES=/peer_id1/https/signaling.example.com/443
-e MINA_WEBNODE_BOOTNODES=/peer_id1/https/signaling.example.com/443,/peer_id2/p2p/peer_id1
```
