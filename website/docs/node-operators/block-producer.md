# Run a block producer

This guide is intended for setting up block producer nodes on **Mina Devnet**
only. Do not use this guide for Mina Mainnet until necessary security audits are
complete.

---

## Prerequisites

Ensure Docker and Docker Compose are installed on your system -
[Docker Installation Guide](../appendix/docker-installation)

## Using Docker Compose

1. **Download the Docker Compose File**

   Create a directory for your block producer and download the docker-compose
   file:

   ```bash
   # Create a directory for your block producer
   mkdir mina-block-producer && cd mina-block-producer

   # Create the working directory for node data
   mkdir mina-workdir

   # Download the block producer docker-compose file (choose one method)
   # Using wget:
   wget https://raw.githubusercontent.com/o1-labs/mina-rust/v0.18.0/docker-compose.block-producer.yml

   # Or using curl:
   curl -O https://raw.githubusercontent.com/o1-labs/mina-rust/v0.18.0/docker-compose.block-producer.yml

   # Create an empty .env file to avoid warnings (optional - has defaults)
   touch .env
   ```

   For the latest development version, replace `v0.18.0` with `develop` in the
   URL.

2. **Verify Docker Image Version** (Optional but recommended)

   Before starting, verify you have the correct version:

   ```bash
   docker run --rm o1labs/mina-rust:latest build-info
   ```

   This shows the exact version, commit hash, and build details.

3. **Prepare Your Keys**

   [Docker Compose](https://github.com/o1-labs/mina-rust/blob/develop/docker-compose.block-producer.yml)
   references `mina-workdir`. It stores a private key and logs for block
   production.

   **Option A: Generate a new key pair (if you don't have one)**

   If you don't have a block producer key, you can generate one using the Mina
   CLI:

   ```bash
   # Generate a new encrypted key pair using Docker
   # This creates the key directly in the mina-workdir/producer-key file
   docker run --rm -v $(pwd)/mina-workdir:/root/.mina o1labs/mina-rust:latest \
     misc mina-encrypted-key "YourPassword" --file /root/.mina/producer-key

   # Fix file permissions to match your local user (recommended)
   sudo chown $(id -u):$(id -g) mina-workdir/producer-key
   ```

   This will create an encrypted private key file at
   `mina-workdir/producer-key`. The key will be encrypted with the password you
   provide.

   <!-- prettier-ignore-start -->

   :::tip File Permissions

   Docker containers typically run as root, so generated files will be owned by
   root. It's recommended to change ownership to your local user using `chown`
   after generation to avoid permission issues when accessing or backing up the
   key file.

   :::
   <!-- prettier-ignore-stop -->

   The command also outputs the public key which you should save for reference:

   ```bash
   # To see just the public key, you can run:
   docker run --rm -v $(pwd)/mina-workdir:/root/.mina o1labs/mina-rust:latest \
     misc mina-encrypted-key "YourPassword" --file /root/.mina/producer-key | grep "Public key:"
   ```

   **Option B: Use an existing key**

   If you already have a block producer key, place it into the `mina-workdir`
   directory and name it `producer-key`:

   ```bash
   cp /path/to/your/private_key mina-workdir/producer-key
   ```

   Replace `/path/to/your/private_key` with the actual path to your private key
   file.

4. **Launch Block Producer**

   Use `MINA_PRIVKEY_PASS` to set the private key password. Optionally, use
   `COINBASE_RECEIVER` to set a different coinbase receiver:

   ```bash
   env COINBASE_RECEIVER="YourWalletAddress" MINA_PRIVKEY_PASS="YourPassword" \
   docker compose -f docker-compose.block-producer.yml up -d --pull always
   ```

   **Configuration Options:**
   - `MINA_RUST_TAG` - Docker image tag for the mina-rust node (default:
     `latest`)
   - `MINA_FRONTEND_TAG` - Docker image tag for the frontend (default: `latest`)
   - `MINA_LIBP2P_EXTERNAL_IP` - Sets your node's external IP address to help
     other nodes find it
   - `MINA_LIBP2P_PORT` - Sets the port for Libp2p communication
   - `COINBASE_RECEIVER` - Wallet address to receive block rewards
   - `MINA_PRIVKEY_PASS` - Password for encrypted private key

   **Examples with different versions:**

   ```bash
   # Use specific version (recommended for production)
   env MINA_RUST_TAG="v1.4.2" MINA_FRONTEND_TAG="v1.4.2" \
   COINBASE_RECEIVER="YourWalletAddress" MINA_PRIVKEY_PASS="YourPassword" \
   docker compose -f docker-compose.block-producer.yml up -d --pull always

   # Use development version (latest features, may be unstable)
   env MINA_RUST_TAG="develop" MINA_FRONTEND_TAG="develop" \
   COINBASE_RECEIVER="YourWalletAddress" MINA_PRIVKEY_PASS="YourPassword" \
   docker compose -f docker-compose.block-producer.yml up -d --pull always
   ```

5. **Go to Dashboard**

   Visit [http://localhost:8070](http://localhost:8070) to monitor sync and
   block production.

## Using Make Command

As an alternative to Docker Compose, you can run the block producer directly
using the Makefile target. This method requires building from source.

### Prerequisites

- Rust toolchain installed
- Git repository cloned and accessible

### Setup and Run

1. **Prepare Your Keys**

   You have two options for setting up your producer key:

   **Option A: Generate a new key pair**

   ```bash
   make generate-block-producer-key
   ```

   This will create a new key pair and save the private key to
   `mina-workdir/producer-key` and the public key to
   `mina-workdir/producer-key.pub`. The command will fail if keys already exist
   to prevent accidental overwriting.

   To generate keys with a password:

   ```bash
   make generate-block-producer-key MINA_PRIVKEY_PASS="YourPassword"
   ```

   To generate keys with a custom filename:

   ```bash
   make generate-block-producer-key PRODUCER_KEY_FILENAME=./path/to/custom-key
   ```

   This will create `./path/to/custom-key` (private) and
   `./path/to/custom-key.pub` (public).

   You can combine both options:

   ```bash
   make generate-block-producer-key \
     PRODUCER_KEY_FILENAME=./path/to/custom-key \
     MINA_PRIVKEY_PASS="YourPassword"
   ```

   **Option B: Use an existing key**

   ```bash
   mkdir -p mina-workdir
   cp /path/to/your/private_key mina-workdir/producer-key
   ```

2. **Run Block Producer**

   For devnet (default):

   ```bash
   make run-block-producer \
       MINA_PRIVKEY_PASS="YourPassword" \
       NETWORK=devnet \
       COINBASE_RECEIVER="YourWalletAddress"
   ```

   For mainnet (when supported):

   ```bash
   make run-block-producer \
       COINBASE_RECEIVER="YourWalletAddress" \
       MINA_PRIVKEY_PASS="YourPassword" \
       NETWORK=mainnet
   ```

   Optional parameters:
   - `MINA_LIBP2P_EXTERNAL_IP` - Sets external IP address
   - `MINA_LIBP2P_PORT` - Sets libp2p communication port
   - `PRODUCER_KEY_FILENAME` - Path to producer key (default:
     `./mina-workdir/producer-key`)

   Example with all options:

   ```bash
   make run-block-producer \
     NETWORK=devnet \
     COINBASE_RECEIVER="YourWalletAddress" \
     MINA_PRIVKEY_PASS="YourPassword" \
     MINA_LIBP2P_EXTERNAL_IP="1.2.3.4" \
     MINA_LIBP2P_PORT="8302"
   ```

3. **Monitor the Node**

   The node will start and listen on port 3000. You can monitor its status by
   checking the console output or connecting a frontend dashboard.

### Access Logs

Logs are stored in `mina-workdir` with filenames like `mina.log.2024-10-14`,
`mina.log.2024-10-15`, etc.

### Provide Feedback

Collect logs from `mina-workdir` and report issues on the
[rust-node-testing](https://discord.com/channels/484437221055922177/1290662938734231552)
discord channel. Include reproduction steps if possible.
