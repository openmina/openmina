```
docker build -t vladsimplestakingcom/mina-light:2.0.0rampup4 -f node/testing/docker/Dockerfile.light node/testing/docker
docker build -t vladsimplestakingcom/mina-light:2.0.0rampup4-focal -f node/testing/docker/Dockerfile.light.focal node/testing/docker
docker build -t vladsimplestakingcom/mina-openmina-builder:latest -f node/testing/docker/Dockerfile.openmina node/testing/docker
docker build -t vladsimplestakingcom/mina-testenv:2.0.0rampup4 -f node/testing/docker/Dockerfile.test node/testing/docker
docker build -t vladsimplestakingcom/mina-debugger:2.0.0rampup4-focal -f node/testing/docker/Dockerfile.debugger node/testing/docker
```
