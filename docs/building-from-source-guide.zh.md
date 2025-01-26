# 如何从源码构建和启动节点

本安装指南已在 Debian 和 Ubuntu 系统上测试过，应该适用于大多数 Linux 发行版。

## 前置要求

Ubuntu 或基于 Debian 的 Linux 发行版，需要安装以下软件包：

- `curl`
- `git`
- `libssl-dev`
- `pkg-config`
- `protobuf-compiler`
- `build-essential`

示例（基于 debian）：

```sh
# 使用 "sudo" 或以 "root" 用户身份运行
sudo apt install curl git libssl-dev pkg-config protobuf-compiler build-essential
```

示例（macOS）：

如果您尚未安装 homebrew：

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

```sh
brew install curl git openssl pkg-config protobuf gcc make
```

## 步骤（适用于基于 Debian 的 Linux 发行版和 macOS）：

打开命令行并输入以下命令：

```sh
# 安装 rustup 并将默认 Rust 工具链设置为 1.80（更新版本也可以）
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.80
# 为当前 shell 配置 rustup
source "$HOME/.cargo/env"
# 克隆 openmina 仓库
git clone https://github.com/openmina/openmina.git
cd openmina/
# 构建并运行节点
cargo run --release -p cli node
```

# 如何启动 UI：

## 前置要求

### 1. Node.js v20.11.1

#### MacOS

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install node@20.11.1
```

#### Linux

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
source ~/.bashrc
nvm install 20.11.1
```

#### Windows

从官方网站下载 [Node.js v20.11.1](https://nodejs.org/)，打开安装程序并按照提示完成安装。

### 2. Angular CLI v16.2.0

```bash
npm install -g @angular/cli@16.2.0
```

### 3. 安装

打开终端并导航到项目根目录

```bash
cd PROJECT_LOCATION/openmina/frontend
```

安装依赖

```bash
npm install
```

## 运行应用

```bash
npm start
```
