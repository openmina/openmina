# Rust Node Block Producer Quick Start Guide

---

## Prerequisites

Ensure Docker and Docker Compose are installed on your system.

- **Docker Installation Guide**: [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)

## 1. Setup Work Directory
Create the main directory and a subdirectory to store your node's keys and logs:

    ```bash
    mkdir -p ~/mina-node/openmina-workdir cd ~/mina-node

    ```

---

## 2. Prepare Your Keys
Place your block producer's private key into the `openmina-workdir` directory and name it `producer-key`:

    ```bash
    cp /path/to/your/private_key producer-key
    ```
Replace `/path/to/your/private_key` with the actual path to your private key file.

---


## 3. Launch Block Producer
Download the Docker Compose file:

    ```bash
    curl -O https://raw.githubusercontent.com/openmina/openmina/main/docker-compose.block-producer.yml
    ```

Launch the block producer with parameters `MINA_PRIVKEY_PASS` to specify the password for the private key, and optional `COINBASE_RECEIVER` to make the coinbase receiver an account other than the producer key:

    ```bash
    env COINBASE_RECEIVER="YourWalletAddress" MINA_PRIVKEY_PASS="YourPassword" \
    docker compose -f docker-compose.block-producer.yml up -d --pull always
    ```
---

## 4. Access Dashboard

Visit [http://localhost:8070](http://localhost:8070) to [monitor sync](localhost:8070/dashboard) and [block production](http://localhost:8070/block-production).

---

## 5. Logs & Node Management

Logs are stored in `openmina-workdir`. 

To view the logs:

    ```bash
    docker compose -f docker-compose.block-producer.yml logs -f
    ```

Restart the node:

    ```bash
    docker compose down && docker compose up -d --pull always
    ```

Update the node to the latest version:

    ```bash
    docker compose pull && docker compose up -d
    ```

---


## 6. Feedback & Troubleshooting

Collect logs from `openmina-workdir` and report issues on the Open Mina Discord channel. Include reproduction steps if possible.

---
