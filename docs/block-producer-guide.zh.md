# 在开发网络上运行区块生产节点

本指南仅适用于在 **Mina 开发网络** 上设置区块生产节点。  
在完成必要的安全审计之前，请勿将本指南用于 Mina 主网。

---

## 前置要求

确保您的系统已安装 Docker 和 Docker Compose - [Docker 安装指南](./docker-installation.zh.md)

## 下载并启动节点

1. **下载最新版本**

- 访问 [Open Mina 发布页面](https://github.com/openmina/openmina/releases)
- 下载最新的 `openmina-vX.Y.Z-docker-compose.zip`
- 解压文件：

  ```bash
  unzip openmina-vX.Y.Z-docker-compose.zip
  cd openmina-vX.Y.Z-docker-compose
  mkdir openmina-workdir
  ```

2. **准备密钥**

   [Docker Compose](../docker-compose.block-producer.yml) 引用 `openmina-workdir`。它存储区块生产所需的私钥和日志。
   将您的区块生产者私钥放入 `openmina-workdir` 目录并命名为 `producer-key`：

   ```bash
   cp /path/to/your/private_key openmina-workdir/producer-key
   ```

   将 `/path/to/your/private_key` 替换为您私钥文件的实际路径。

3. **启动区块生产者**

   使用 `MINA_PRIVKEY_PASS` 设置私钥密码。可选择使用 `COINBASE_RECEIVER` 设置不同的币基接收地址：

   ```bash
   env COINBASE_RECEIVER="您的钱包地址" MINA_PRIVKEY_PASS="您的密码" \
   docker compose -f docker-compose.block-producer.yml up -d --pull always
   ```

   可选参数：

   `OPENMINA_LIBP2P_EXTERNAL_IP` 设置您节点的外部 IP 地址，以帮助其他节点找到它。

   `OPENMINA_LIBP2P_PORT` 设置 Libp2p 通信的端口。

4. **访问仪表盘**

   访问 [http://localhost:8070](http://localhost:8070) 来[监控同步状态](http://localhost:8070/dashboard)和[区块生产](http://localhost:8070/block-production)。

### 访问日志

日志存储在 `openmina-workdir` 中，文件名格式如 `openmina.log.2024-10-14`、`openmina.log.2024-10-15` 等。

### 提供反馈

收集 `openmina-workdir` 中的日志，并在 [rust-node-testing](https://discord.com/channels/484437221055922177/1290662938734231552) Discord 频道报告问题。如果可能，请包含复现步骤。
