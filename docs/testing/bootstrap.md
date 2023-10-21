# Bootstrapping Openmina

## Bootstrap using Berkeleynet

This test is focused on ensuring that the latest Openmina build is able to
bootstrap against Berkeleynet. It is executed on a daily basis.

The node's HTTP port is accessible at http://1.k8.openmina.com:31001.

These are the main steps and checks.

First, it performs some checks on the instance deployed previously:
- Node is in sync state
- Node's best tip is the one that of berkeleynet

Then it deploys the new instance of Openmina and waits until it is bootstrapped
(with a timeout of 10 minutes). After that. it performs the following checks:

- The node's best tip is the same as in berkeleynet
- There were no restarts for the openmina container


See the [Openmina Daily](../../.github/workflows/daily.yaml) workflow file for
further details.

