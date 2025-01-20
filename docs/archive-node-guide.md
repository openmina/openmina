# Run Archive Node on Devnet

This guide is intended for setting up archive nodes on **Mina Devnet** only. Do not use this guide for Mina Mainnet

## Prerequisites

Ensure Docker and Docker Compose are installed on your system - [Docker Installation Guide](./docker-installation.md)

## Docker compose setup

The compose file sets up a PG database, the archiver process and the openmina node. The archiver process is responsible for storing the blocks in the database by receiving the blocks from the openmina node. We start the archive mode in openmina by setting the `--archive-mode` flag to the address fo archiver process. See [docker-compose.archive.devnet.yml](../docker-compose.archive.devnet.yml) for more details.

## Starting the setup

```bash
docker compose -f docker-compose.archive.devnet.yml up -d
```
