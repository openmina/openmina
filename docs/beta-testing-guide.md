# Rust Node Block Producer Quick Start Guide

This guide is intended for setting up block producer nodes on **Mina Devnet** only.  
Do not use this guide for Mina Mainnet until necessary security audits are complete.


---

### 0. Prerequisites

Ensure Docker and Docker Compose are installed on your system - [Docker Installation Guide](./docker-installation.md)

### 1. Setup Directories
Create the main directory and a subdirectory `openmina-workdir`. [Docker Compose](../docker-compose.block-producer.yml) references  `openmina-workdir`. It stores a private key and logs for block production. 

```bash
mkdir -p ~/mina-node/openmina-workdir cd ~/mina-node
```

### 2. Prepare Your Keys
Place your block producer's private key into the `openmina-workdir` directory and name it `producer-key`:

```bash
cp /path/to/your/private_key producer-key
```
Replace `/path/to/your/private_key` with the actual path to your private key file.

### 3. Launch Block Producer
Download the Docker Compose file:

```bash
curl -O https://raw.githubusercontent.com/openmina/openmina/main/docker-compose.block-producer.yml
```

Launch the block producer with MINA_PRIVKEY_PASS to set the private key password. Optionally, use COINBASE_RECEIVER to set a different coinbase receiver:

```bash
env COINBASE_RECEIVER="YourWalletAddress" MINA_PRIVKEY_PASS="YourPassword" \
docker compose -f docker-compose.block-producer.yml up -d --pull always
```

### 4. Access Dashboard

Visit [http://localhost:8070](http://localhost:8070) to [monitor sync](http://localhost:8070/dashboard) and [block production](http://localhost:8070/block-production).

### 5. Logs

Logs are stored in `openmina-workdir` with filenames like openmina.log.2024-10-14, openmina.log.2024-10-15, etc.

### 6. Feedback

Collect logs from `openmina-workdir` and report issues on the [rust-node-testing](https://discord.com/channels/484437221055922177/1290662938734231552) discord channel. Include reproduction steps if possible.

