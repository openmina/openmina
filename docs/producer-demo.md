# Producer Demo Launch Guide with Docker

This demo showcases the block production capabilities of OpenMina nodes within a private network. It launches three OpenMina nodes on your local machine, operating in a private network environment. For the purpose of this demonstration, block proofs are disabled. This setup allows you to observe block production immediately, without the need to wait for your account to be included in the staking ledger on testnets.

## Known issues

The nodes in the demo can sometimes become unresponsive, especially immediately after startup. If the frontend is unable to contact the nodes a few minutes after launching the Docker Compose setup, restart the setup and try again.

## Table of Contents
1. [Prerequisites](#prerequisites)
    - [Docker Installation on Debian-based Linux](#docker-installation-on-debian-based-linux)
    - [Docker Installation on Windows](#docker-installation-on-windows)
    - [Docker Installation on macOS](#docker-installation-on-macos)
2. [Running the Producer Demo](#running-the-producer-demo)

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

Once pre-requisites for your OS have been completed, follow these steps (they are the same for Debian-based Linux, Windows and MacOS):

### Setup Option 1: Download Docker Compose Files from the Release

1. **Download the Docker Compose files:**
   - Go to the [Releases page](https://github.com/openmina/openmina/releases) of this repository.
   - Download the latest `openmina-vX.Y.Z-docker-compose.zip` (or `.tar.gz`) file corresponding to the release version (available since v0.8.0).

2. **Extract the files:**
   - Unzip or untar the downloaded file:
     ```bash
     unzip openmina-vX.Y.Z-docker-compose.zip
     ```
     or
     ```bash
     tar -xzvf openmina-vX.Y.Z-docker-compose.tar.gz
     ```
   - Replace `vX.Y.Z` with the actual release version you downloaded.

3. **Navigate to the extracted directory:**
   ```bash
   cd openmina-vX.Y.Z-docker-compose
   ```

### Setup Option 2: Clone the Repository

1. **Clone this repository:**
   ```bash
   git clone https://github.com/openmina/openmina.git
   ```

2. **Navigate to the repository:**
   ```bash
   cd openmina
   ```

### Launch

**Run the following command to start the demo:**
   ```bash
   docker compose -f docker-compose.local.producers.yml up --pull always --force-recreate
   ```

And finally:

**Open your browser and visit [http://localhost:8070](http://localhost:8070)**

You should see the following screen:

![producer-demo](https://github.com/user-attachments/assets/f0ccc36e-0ee8-4284-a8d7-de0f9a3397d6)
