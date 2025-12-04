```
docker build -t vladsimplestakingcom/mina-light:2.0.0rampup4 -f tools/testing/docker/Dockerfile.light tools/testing/docker
docker build -t vladsimplestakingcom/mina-light:2.0.0rampup4-focal -f tools/testing/docker/Dockerfile.light.focal tools/testing/docker
docker build -t vladsimplestakingcom/mina-openmina-builder:focal -f tools/testing/docker/Dockerfile.openmina tools/testing/docker
docker build -t vladsimplestakingcom/mina-testenv:2.0.0rampup4-focal -f tools/testing/docker/Dockerfile.test tools/testing/docker
docker build -t vladsimplestakingcom/mina-debugger:2.0.0rampup4-focal -f tools/testing/docker/Dockerfile.debugger tools/testing/docker
```
