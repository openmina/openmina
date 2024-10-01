# Test Alpha Rust Node on Devnet

This guide will walk you through running the **Alpha Rust Node** on Devnet using Docker. Follow these steps to set up the node and provide feedback on this Alpha release.

## 1. Prerequisites

Ensure you have **Docker** installed. For installation instructions specific to your operating system, follow the steps outlined in the **Producer Demo Launch Guide**:

- [Docker Installation Guide for Linux, macOS, and Windows](https://github.com/openmina/openmina/blob/main/docs/producer-demo.md#prerequisites)

## 2. Download & Start the Node

### Steps:

1. **Download the Latest Alpha Release**:
   - Visit the [Open Mina Releases](https://github.com/openmina/openmina/releases).
   - Download the latest `openmina-vX.Y.Z-docker-compose.zip` (Alpha release).

2. **Extract the Files**:
   ```bash
   unzip openmina-vX.Y.Z-docker-compose.zip
   cd openmina-vX.Y.Z-docker-compose
   ```

3. **Start the Node on Devnet and Save Logs**:
   Start the node and save the logs for later analysis:
   ```bash
   docker compose up --pull always && docker compose logs > openmina-node.log
   ```

4. **Access the Dashboard**:
   Open `http://localhost:8070` in your browser.The dashboard will show the syncing process in real time. Syncing Steps:

  - **Connecting to Peers:** The node connects to peers. You’ll see the number of connected, connecting, and disconnected peers grow.
  - **Fetching Ledgers:** The node downloads key data: Staking ledger, Next epoch ledger, and Snarked ledger. Progress bars show the download status.
  - **Fetching & Applying Blocks:** The node downloads recent blocks to match the network’s current state. The dashboard tracks how many blocks are fetched and applied.
  - **Synced:** The node is fully synced with the network and ready to operate.

## 3. Logs

Monitoring and troubleshooting:

### View Real-Time Logs:
```bash
docker compose logs -f
```

### Save Logs to a File:
Use the following command if you want to save the current logs for later analysis, e.g., for debugging or reporting issues:
```bash
docker compose logs > openmina-node.log
```

## 4. Troubleshooting

### Restart the Node:
If the node becomes unresponsive or fails to start, restart the setup:
```bash
docker compose down
docker compose up --pull always
```

## 5. Provide Feedback

This Alpha release is for testing purposes. Your feedback is essential. Follow these steps to report any issues:

1. **Collect Logs**: Use the commands above to save logs.
2. **Join Discord**: [discord.gg/your-discord-invite-link](discord link).
3. **Describe the Issue**: Briefly explain the problem and steps to reproduce it.
4. **Attach Logs**: Include the `openmina-node.log` file when reporting.

**Note**: Including a screenshot of the dashboard when reporting issues can provide helpful context. The dashboard shows key details about syncing, peer connections, and block application that may assist in diagnosing the issue.
