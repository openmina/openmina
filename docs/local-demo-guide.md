# Run Block Producers on local network

Once you have completed the [pre-requisites](./docs/docker-installation.md) for your operating system, follow these steps:

## Setup Option 1: Download Docker Compose Files from the Release

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

## Setup Option 2: Clone the Repository

1. **Clone this repository:**

   ```bash
   git clone https://github.com/openmina/openmina.git
   ```

2. **Navigate to the repository:**
   ```bash
   cd openmina
   ```

## Launch

**Run the following command to start the demo:**

```bash
docker compose -f docker-compose.local.producers.yml up --pull always --force-recreate
```

And finally:

**Open your browser and visit [http://localhost:8070](http://localhost:8070)**

You should see the following screen:

![producer-demo](https://github.com/user-attachments/assets/f0ccc36e-0ee8-4284-a8d7-de0f9a3397d6)
