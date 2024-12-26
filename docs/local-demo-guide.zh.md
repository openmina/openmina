# 在本地网络上运行区块生产者

在完成您操作系统的[前置要求](./docs/docker-installation.zh.md)后，请按照以下步骤操作：

## 设置选项 1：从发布版本下载 Docker Compose 文件

1. **下载 Docker Compose 文件：**

   - 访问本仓库的[发布页面](https://github.com/openmina/openmina/releases)
   - 下载最新的 `openmina-vX.Y.Z-docker-compose.zip`（或 `.tar.gz`）文件，对应发布版本（从 v0.8.0 版本开始提供）

2. **解压文件：**

   - 解压下载的文件：
     ```bash
     unzip openmina-vX.Y.Z-docker-compose.zip
     ```
     或
     ```bash
     tar -xzvf openmina-vX.Y.Z-docker-compose.tar.gz
     ```
   - 将 `vX.Y.Z` 替换为您下载的实际发布版本号

3. **进入解压后的目录：**
   ```bash
   cd openmina-vX.Y.Z-docker-compose
   ```

## 设置选项 2：克隆仓库

1. **克隆此仓库：**

   ```bash
   git clone https://github.com/openmina/openmina.git
   ```

2. **进入仓库目录：**
   ```bash
   cd openmina
   ```

## 启动

**运行以下命令启动演示：**

```bash
docker compose -f docker-compose.local.producers.yml up --pull always --force-recreate
```

最后：

**在浏览器中访问 [http://localhost:8070](http://localhost:8070)**

您应该能看到以下界面：

![区块生产者演示](https://github.com/user-attachments/assets/f0ccc36e-0ee8-4284-a8d7-de0f9a3397d6)
