# Producer Demo Launch Guide with Docker

This demo showcases the block production capabilities of OpenMina nodes within a private network. It launches three OpenMina nodes on your local machine, operating in a private network environment. For the purpose of this demonstration, block proofs are disabled. This setup allows you to observe block production immediately, without the need to wait for your account to be included in the staking ledger on testnets.

## Table of Contents
1. [Prerequisites](#prerequisites)
    - [Docker Installation on Debian-based Linux](#docker-installation-on-debian-based-linux)
    - [Docker Installation on Windows](#docker-installation-on-windows)
    - [Docker Installation on macOS](#docker-installation-on-macos)
2. [Running the Producer Demo](#running-the-producer-demo)
    - [Running on Debian-based Linux](#running-on-debian-based-linux)
    - [Running on Windows](#running-on-windows)
    - [Running on macOS](#running-on-macos)

## Prerequisites

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

### Docker Installation on Windows

1. **Download Docker Desktop for Windows from the [Docker Website](https://www.docker.com/products/docker-desktop/).**

2. **Run the installer and follow the instructions.**

3. **After installation, Docker will start automatically. Verify that Docker Desktop is running by checking the Docker icon in your system tray.**

### Docker Installation on macOS

1. **Download Docker Desktop for Mac from the [Docker Website](https://www.docker.com/products/docker-desktop/).**

2. **Open the downloaded `.dmg` file and drag Docker to the Applications folder.**

3. **Open Docker from the Applications folder and follow the installation steps.**

4. **Open up a terminal window by using the shortcut Command + Space and type in terminal**

5. **Verify the installation by running the following command in your terminal:**
    ```sh
    docker --version
    ```

## Running the Producer Demo

### Running on Debian-based Linux

1. **Clone this repository:**
    ```bash
    git clone https://github.com/openmina/openmina.git
    ```

2. **Navigate to the repository:**

    ```bash
    cd openmina
    ```

3. **Run the following command to start the demo:**
    ```sh
    docker compose -f docker-compose.local.producers.yml up
    ```

4. **Open you browser and visit http://localhost:8070**

### Running on Windows

1. **Open the command prompt by pressing the windows button on your keyboard and type in command prompt**

2. **Clone this repository:**
    ```bash
    git clone https://github.com/openmina/openmina.git
    ```

3. **Navigate to the repository:**

    ```bash
    cd openmina
    ```

4. **Run the following command to start the demo:**
    ```sh
    docker compose -f docker-compose.local.producers.yml up
    ```

5. **Open you browser and visit http://localhost:8070**

### Running on macOS

1. **Open a terminal by pressing command + space on your keyboard and type in terminal**

2. **Clone this repository:**
    ```bash
    git clone https://github.com/openmina/openmina.git
    ```

3. **Navigate to the repository:**

    ```bash
    cd openmina
    ```

4. **Run the following command to start the demo:**
    ```sh
    docker compose -f docker-compose.local.producers.yml up
    ```

5. **Open you browser and visit http://localhost:8070**
