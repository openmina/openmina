# 在开发网络上运行非出块节点

本指南将指导您使用 Docker 在开发网络上运行 **Alpha Rust 节点**。按照以下步骤设置节点并为这个 Alpha 版本[提供反馈](#4-提供反馈)。

## 1. 前置要求

确保您已安装 **Docker**：

- [Linux、macOS 和 Windows 的 Docker 安装指南](./docker-installation.md)

## 2. 下载并启动节点

1. **下载最新版本**：

   - 访问 [Open Mina 发布页面](https://github.com/openmina/openmina/releases)
   - 下载最新的 `openmina-vX.Y.Z-docker-compose.zip`

2. **解压文件**：

   ```bash
   unzip openmina-vX.Y.Z-docker-compose.zip
   cd openmina-vX.Y.Z-docker-compose
   ```

   额外的可选参数：

   `OPENMINA_LIBP2P_EXTERNAL_IP` 设置您节点的外部 IP 地址，以帮助其他节点找到它。

   `OPENMINA_LIBP2P_PORT` 设置 Libp2p 通信的端口。

3. **在开发网络上启动节点并保存日志**：
   启动节点并保存日志以供后续分析：

   ```bash
   docker compose up --pull always && docker compose logs > openmina-node.log
   ```

4. **访问仪表板**：
   在浏览器中打开 `http://localhost:8070`。

   仪表板将实时显示同步过程。
   <img width="1417" alt="image" src="https://github.com/user-attachments/assets/d9a5f5b3-522f-479b-9829-37402c63bb98">

   > **1. 连接节点：** 节点与其他对等节点建立连接。您将看到已连接、正在连接和断开连接的节点数量增长。
   >
   > **2. 获取账本：** 节点下载关键数据：权益账本、下一轮账本和已验证账本。进度条显示下载状态。
   >
   > **3. 获取并应用区块：** 节点下载最近的区块以匹配网络的当前状态。仪表板跟踪已获取和应用的区块数量。

## 3. 监控和故障排除

### 检查保存的日志

如果您已将日志保存到文件中，可以使用 tail 或类似工具查看：

```bash
tail -f openmina-node.log
```

### 重启节点：

如果节点无响应或无法启动，请重启设置：

```bash
docker compose down
docker compose up --pull always
```

## 4. 提供反馈

此 Alpha 版本用于测试目的。您的反馈至关重要。按照以下步骤报告任何问题：

1. **收集日志**：使用[上述命令保存日志](#2-下载并启动节点)
2. **访问 Discord**：[Open Mina Discord 频道](https://discord.com/channels/484437221055922177/1290662938734231552/1290667779317305354)
3. **描述问题**：简要说明问题和重现步骤
4. **附上日志**：Discord 允许上传最大 25MB 的文件。如果您的日志更大，请使用 Google Drive 或类似服务
5. **包含截图**：仪表板截图提供了节点状态的详细信息，便于诊断问题
