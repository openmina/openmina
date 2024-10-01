# Test Alpha Rust Node on Devnet

This guide will walk you through running the **Alpha Rust Node** on Devnet using Docker. Follow these steps to set up the node and [Provide Feedback](#4-provide-feedback) on this Alpha release.

## 1. Prerequisites

Ensure you have **Docker** installed. For installation instructions specific to your operating system, follow the steps outlined in the **Producer Demo Launch Guide**:

- [Docker Installation Guide for Linux, macOS, and Windows](https://github.com/openmina/openmina/blob/main/docs/producer-demo.md#prerequisites)

## 2. Download & Start the Node

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
   Open `http://localhost:8070` in your browser.
   
   The dashboard will show the syncing process in real time. 
   <img width="1417" alt="image" src="https://github.com/user-attachments/assets/d9a5f5b3-522f-479b-9829-37402c63bb98">
   
   >**1. Connecting to Peers:** The node connects to peers. You’ll see the number of connected, connecting, and disconnected peers grow.
   >
   >**2. Fetching Ledgers:** The node downloads key data: Staking ledger, Next epoch ledger, and Snarked ledger. Progress bars show the download status.
   >
   >**3. Fetching & Applying Blocks:** The node downloads recent blocks to match the network’s current state. The dashboard tracks how many blocks are fetched and applied.

## 3. Monitoring and troubleshooting

### Inspecting Saved Logs
If you’ve saved logs to a file, you can use tail or similar tools to view them:

```bash
tail -f openmina-node.log
```

### Restart the Node:
If the node becomes unresponsive or fails to start, restart the setup:
```bash
docker compose down
docker compose up --pull always
```

## 4. Provide Feedback

This Alpha release is for testing purposes. Your feedback is essential. Follow these steps to report any issues:

1. **Collect Logs**: Use the [commands above to save logs](#2-download--start-the-node)
2. **Visit Discord**: [Open Mina Discord Channel](https://discord.com/channels/484437221055922177/1290662938734231552/1290667779317305354)
3. **Describe the Issue**: Briefly explain the problem and steps to reproduce it
4. **Attach Logs**: Discord allows files up to 25MB. If your logs are larger, use Google Drive or similar
5. **Include a Screenshot**: A dashboard screenshot provides details about node status, making it easier to diagnose the issue
