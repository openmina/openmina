# Docker Installation Guide

## Installation for
    - [Debian-based Linux](#docker-installation-on-debian-based-linux)
    - [Windows](#docker-installation-on-windows)
    - [macOS](#docker-installation-on-macos)

### Docker Installation on Debian-based Linux

1. **Set up Docker's apt repository:**

    ```bash
    # Add Docker's official GPG key:
    sudo apt-get update
    sudo apt-get install ca-certificates curl
    sudo install -m 0755 -d /etc/apt/keyrings
    sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
    sudo chmod a+r /etc/apt/keyrings/docker.asc

    # Add the repository to Apt sources:
    echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
    $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
    sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
    sudo apt-get update
    ```

2. **Install the Docker packages:**

    ```bash
    sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    ```

3. **Add your user to the `docker` group:**

    ```bash
    sudo usermod -aG docker $USER
    newgrp docker
    ```

4. **Verify the installation:**

    ```bash
    docker run hello-world
    ```

---

### Docker Installation on Windows

1. **Download and Install [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop/).**

2. **Ensure Docker Desktop is running (check the system tray icon).**

---

### Docker Installation on macOS

1. **Download and Install [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop/).**
2. **Verify the installation in Terminal:**

    ```bash
    docker --version
    ```