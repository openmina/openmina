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

| Environment Variable        | Required? | Description                                                            |
| :-------------------------- | :-------: | :--------------------------------------------------------------------- |
| `MINA_FRONTEND_ENVIRONMENT` |    Yes    | Must be set to `webnode` to run the frontend in webnode mode           |
| `MINA_WEBNODE_SEED_URLS`    |    No     | A comma-separated list of http(s) URLs to fetch initial P2P Seeds from |
| `MINA_WEBNODE_BOOTNODES`    |    No     | A comma-separated list of initial peers in WebRTC-Multiaddrish format  |
