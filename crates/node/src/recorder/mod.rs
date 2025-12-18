#[allow(clippy::module_inception)]
mod recorder;
pub use recorder::Recorder;

mod replayer;
pub use replayer::StateWithInputActionsReader;

use std::{
    borrow::Cow,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{p2p::identity::SecretKey as P2pSecretKey, Action, ActionKind, ActionWithMeta, State};

fn initial_state_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("initial_state.postcard")
}

fn actions_path<P: AsRef<Path>>(path: P, file_index: usize) -> PathBuf {
    path.as_ref()
        .join(format!("actions_{}.postcard", file_index))
}

#[derive(Serialize, Deserialize)]
pub struct RecordedInitialState<'a> {
    pub rng_seed: [u8; 32],
    pub p2p_sec_key: P2pSecretKey,
    pub state: Cow<'a, State>,
}

impl RecordedInitialState<'_> {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> postcard::Result<()> {
        postcard::to_io(self, writer).and(Ok(()))
    }

    pub fn decode(encoded: &[u8]) -> postcard::Result<Self> {
        postcard::from_bytes(encoded)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordedActionWithMeta<'a> {
    pub kind: ActionKind,
    pub meta: redux::ActionMeta,
    pub action: Option<Cow<'a, Action>>,
}

impl RecordedActionWithMeta<'_> {
    pub fn encode(&self) -> postcard::Result<Vec<u8>> {
        postcard::to_stdvec(self)
    }

    pub fn decode(encoded: &[u8]) -> postcard::Result<Self> {
        postcard::from_bytes(encoded)
    }

    #[expect(
        clippy::result_large_err,
        reason = "The error variant is the same Self which is moved in; shouldn't blow up the stack"
    )]
    pub fn as_action_with_meta(self) -> Result<ActionWithMeta, Self> {
        if let Some(action) = self.action {
            let action = action.into_owned();
            Ok(self.meta.with_action(action))
        } else {
            Err(self)
        }
    }
}

impl<'a> From<&'a ActionWithMeta> for RecordedActionWithMeta<'a> {
    fn from(value: &'a ActionWithMeta) -> Self {
        Self {
            kind: value.action().kind(),
            meta: value.meta().clone(),
            action: Some(Cow::Borrowed(value.action())),
        }
    }
}

impl From<(ActionKind, redux::ActionMeta)> for RecordedActionWithMeta<'static> {
    fn from((kind, meta): (ActionKind, redux::ActionMeta)) -> Self {
        Self {
            kind,
            meta,
            action: None,
        }
    }
}
