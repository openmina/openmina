# Bootstrapping Openmina

## Bootstrap using Berkeleynet

This test is focused on ensuring that the latest Openmina build is able to
bootstrap against Berkeleynet. It is executed on daily basis.

The node's HTTP port is accessible as http://1.k8.openmina.com:31001.

These are the main steps and checks.

First, it performs some checks on the instance deployed previously:
- Node is in sync state
- Node's best tip is the one that of berkeleynet

Then it deploys the new instance of Openmina and waits until it is bootstrapped
(with 10 minutes timeout). After that it performs the following checks:

- Node's best tip is the one that of berkeleynet
- There were no restarts for openmina container


See [Openmina Daily](../../.github/workflows/daily.yaml) workflow file for
further details.

