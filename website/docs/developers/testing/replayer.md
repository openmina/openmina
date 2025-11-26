---
title: Replayer
description: Record and replay node execution for debugging
sidebar_position: 8
---

import CodeBlock from "@theme/CodeBlock"; import Tabs from "@theme/Tabs"; import
TabItem from "@theme/TabItem"; import RecordNode from
"!!raw-loader!../scripts/replayer/record-node.sh"; import ReplayNode from
"!!raw-loader!../scripts/replayer/replay-node.sh"; import RecordWithCustomDir
from "!!raw-loader!../scripts/replayer/record-with-custom-dir.sh"; import
ReplayIgnoreMismatch from
"!!raw-loader!../scripts/replayer/replay-ignore-mismatch.sh"; import
ReplayDynamicEffects from
"!!raw-loader!../scripts/replayer/replay-dynamic-effects.sh"; import
RecordNodeDocker from "!!raw-loader!../scripts/replayer/record-node-docker.sh";
import ReplayNodeDocker from
"!!raw-loader!../scripts/replayer/replay-node-docker.sh"; import
RecordWithCustomDirDocker from
"!!raw-loader!../scripts/replayer/record-with-custom-dir-docker.sh"; import
ReplayIgnoreMismatchDocker from
"!!raw-loader!../scripts/replayer/replay-ignore-mismatch-docker.sh"; import
ReplayDynamicEffectsDocker from
"!!raw-loader!../scripts/replayer/replay-dynamic-effects-docker.sh";

# Replayer

The replayer is a specialized debugging tool that enables deterministic
recording and replay of node execution. It captures the complete state machine
behavior of a running node, allowing developers to reproduce and debug issues
with perfect accuracy.

## What is a replayer node?

A replayer node is not a separate node type, but rather a mode of operation that
allows you to:

- **Record execution**: Capture initial state and all input actions during node
  operation
- **Deterministic replay**: Reproduce the exact sequence of state transitions
  offline
- **Debug issues**: Analyze problematic behavior by replaying recorded execution
- **Verify behavior**: Ensure state machine transitions remain consistent across
  code changes

### How it works

The replayer operates in two phases:

#### 1. Recording phase

During normal node operation with recording enabled:

- Initial node state is serialized to disk
- All input actions (events that trigger state changes) are logged with metadata
- Effect actions (side effects dispatched by reducers) are tracked
- Data is stored in the `recorder/` directory within the working directory

#### 2. Replay phase

During replay:

- Initial state is loaded from the recording
- Input actions are dispatched in the exact order they occurred
- The replayer verifies that effect actions match the recording
- Any mismatches indicate non-deterministic behavior or code changes

### Use cases

- **Bug reproduction**: Capture a failing node execution and replay it locally
  for debugging
- **Regression testing**: Ensure state machine behavior remains consistent after
  code changes
- **Performance analysis**: Analyze state transitions without network I/O
  overhead
- **CI/CD validation**: Automated testing of recorded scenarios in continuous
  integration

<!-- prettier-ignore-start -->

:::note Deterministic behavior requirement

The replayer requires that the node's state machine is deterministic. Any
non-deterministic behavior (random number generation without proper seeding,
system time calls, etc.) will cause replay validation to fail.

The Mina node architecture uses a seeded RNG and controlled time sources to
ensure deterministic behavior.

:::

<!-- prettier-ignore-stop -->

---

## Recording node execution

To record a node's execution, use the `--record` flag with the
`state-with-input-actions` mode.

### Basic recording

<Tabs groupId="installation-method">
<TabItem value="source" label="From Source">

<CodeBlock language="bash" title="website/docs/developers/scripts/replayer/record-node.sh">
  {RecordNode}
</CodeBlock>

</TabItem>
<TabItem value="docker" label="Using Docker">

<CodeBlock language="bash" title="website/docs/developers/scripts/replayer/record-node-docker.sh">
  {RecordNodeDocker}
</CodeBlock>

</TabItem>
</Tabs>

This will:

- Start a node connected to devnet
- Record all state and actions to the `recorder/` directory
- Run until manually stopped (Ctrl+C)

### Recording with custom directory

<Tabs groupId="installation-method">
<TabItem value="source" label="From Source">

<CodeBlock language="bash"
title="website/docs/developers/scripts/replayer/record-with-custom-dir.sh"

> {RecordWithCustomDir} </CodeBlock>

</TabItem>
<TabItem value="docker" label="Using Docker">

<CodeBlock language="bash"
title="website/docs/developers/scripts/replayer/record-with-custom-dir-docker.sh"

> {RecordWithCustomDirDocker} </CodeBlock>

</TabItem>
</Tabs>

### Recording options

The `--record` parameter accepts the following values:

- **`none`**: No recording (default)
- **`state-with-input-actions`**: Records initial state and all input actions

### Recorded data structure

When recording is enabled, data is stored in the following structure:

```
<work-dir>/recorder
 ├──actions_1.postcard
 ├──actions_2.postcard
 ├──actions_3.postcard
 |──actions_4.postcard
 ├──actions_5.postcard
 ├──actions_6.postcard
 ├──actions_7.postcard
 |──actions_8.postcard
 └──initial_state.postcard
```

<!-- prettier-ignore-start -->

:::warning Storage considerations

Recording can generate significant data over time. Monitor disk space usage when
running with recording enabled for extended periods.

:::

<!-- prettier-ignore-stop -->

---

## Replaying recorded execution

Once you have recorded node execution, you can replay it using the `mina replay`
command.

### Basic replay

<Tabs groupId="installation-method">
<TabItem value="source" label="From Source">

<CodeBlock language="bash" title="website/docs/developers/scripts/replayer/replay-node.sh">
  {ReplayNode}
</CodeBlock>

</TabItem>
<TabItem value="docker" label="Using Docker">

<CodeBlock language="bash" title="website/docs/developers/scripts/replayer/replay-node-docker.sh">
  {ReplayNodeDocker}
</CodeBlock>

</TabItem>
</Tabs>

This will:

- Load the initial state from the recording
- Dispatch all recorded actions in order
- Verify that effect actions match the recording
- Exit when all actions have been replayed

### Replay with build environment checking

By default, replay validates that the recorded build environment matches the
current build. To ignore mismatches (useful when testing code changes):

<Tabs groupId="installation-method">
<TabItem value="source" label="From Source">

<CodeBlock language="bash"
title="website/docs/developers/scripts/replayer/replay-ignore-mismatch.sh"

> {ReplayIgnoreMismatch} </CodeBlock>

</TabItem>
<TabItem value="docker" label="Using Docker">

<CodeBlock language="bash"
title="website/docs/developers/scripts/replayer/replay-ignore-mismatch-docker.sh"

> {ReplayIgnoreMismatchDocker} </CodeBlock>

</TabItem>
</Tabs>

### Replay with dynamic effects

For advanced debugging, you can inject custom effect handlers during replay:

<Tabs groupId="installation-method">
<TabItem value="source" label="From Source">

<CodeBlock language="bash"
title="website/docs/developers/scripts/replayer/replay-dynamic-effects.sh"

> {ReplayDynamicEffects} </CodeBlock>

</TabItem>
<TabItem value="docker" label="Using Docker">

<CodeBlock language="bash"
title="website/docs/developers/scripts/replayer/replay-dynamic-effects-docker.sh"

> {ReplayDynamicEffectsDocker} </CodeBlock>

</TabItem>
</Tabs>

Custom effects allow you to:

- Inspect state at specific points during replay
- Modify behavior for debugging purposes
- Hot-reload the effects library without restarting replay
