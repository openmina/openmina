## `openmina-node`
Combines all state machines of the node into one state machine, which
has all the logic of the node, except services.

Services are abstracted away, so the node's core logic can be run with
any arbitrary service which implements [Service](src/service.rs) trait. That way we avoid
core logic being platform-dependant and enable better/easier testing.

**NOTE:** Services are mostly just IO or computationally heavy tasks that
we want to run in another thread.

---

[Details regarding architecture](../ARCHITECTURE.md)
