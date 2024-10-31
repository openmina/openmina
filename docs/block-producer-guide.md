# Rust Node Block Production Testing on Devnet

This guide is intended for setting up block producer nodes on **Mina Devnet** only.  
Do not use this guide for Mina Mainnet until necessary security audits are complete.

---

## Prerequisites

Ensure Docker and Docker Compose are installed on your system - [Docker Installation Guide](./docker-installation.md)

## Download & Start the Node

1. **Download the Latest Release**

- Visit the [Open Mina Releases](https://github.com/openmina/openmina/releases)
- Download the latest `openmina-vX.Y.Z-docker-compose.zip`
- Extract the Files:

  ```bash
  unzip openmina-vX.Y.Z-docker-compose.zip
  cd openmina-vX.Y.Z-docker-compose
  mkdir openmina-workdir
  ```

2. **Prepare Your Keys**

   [Docker Compose](../docker-compose.block-producer.yml) references `openmina-workdir`. It stores a private key and logs for block production.
   Place your block producer's private key into the `openmina-workdir` directory and name it `producer-key`:

   ```bash
   cp /path/to/your/private_key openmina-workdir/producer-key
   ```

   Replace `/path/to/your/private_key` with the actual path to your private key file.

3. **Launch Block Producer**

   Use `MINA_PRIVKEY_PASS` to set the private key password. Optionally, use `COINBASE_RECEIVER` to set a different coinbase receiver:

   ```bash
   env COINBASE_RECEIVER="YourWalletAddress" MINA_PRIVKEY_PASS="YourPassword" \
   docker compose -f docker-compose.block-producer.yml up -d --pull always
   ```

4. **Go to Dashboard**

   Visit [http://localhost:8070](http://localhost:8070) to [monitor sync](http://localhost:8070/dashboard) and [block production](http://localhost:8070/block-production).

### Access Logs

Logs are stored in `openmina-workdir` with filenames like `openmina.log.2024-10-14`, `openmina.log.2024-10-15`, etc.

### Provide Feedback

Collect logs from `openmina-workdir` and report issues on the [rust-node-testing](https://discord.com/channels/484437221055922177/1290662938734231552) discord channel. Include reproduction steps if possible.
