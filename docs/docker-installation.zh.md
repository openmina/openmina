# Docker 安装指南

## 适用于以下系统的安装指南

- [基于 Debian 的 Linux 系统](#基于-debian-的-linux-系统安装-docker)
- [Windows 系统](#windows-系统安装-docker)
- [macOS 系统](#macos-系统安装-docker)

## 基于 Debian 的 Linux 系统安装 Docker

1. 设置 Docker 的 apt 仓库：

    ```bash
    # 添加 Docker 的官方 GPG 密钥：
    sudo apt-get update
    sudo apt-get install ca-certificates curl
    sudo install -m 0755 -d /etc/apt/keyrings
    sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
    sudo chmod a+r /etc/apt/keyrings/docker.asc

    # 将仓库添加到 Apt 源：
    echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
    $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
    sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
    sudo apt-get update
    ```

2. 安装 Docker 软件包：

    ```bash
    sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    ```

3. 将当前用户添加到 `docker` 用户组：

    ```bash
    sudo usermod -aG docker $USER
    newgrp docker
    ```

4. 验证安装：

    ```bash
    docker run hello-world
    ```

---

## Windows 系统安装 Docker

1. 下载并安装 [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop/)。

2. 确保 Docker Desktop 正在运行（检查系统托盘图标）。

---

## macOS 系统安装 Docker

1. 下载并安装 [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop/)。
2. 在终端中验证安装：

    ```bash
    docker --version
    ```
