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

use crate::{Action, ActionKind, ActionWithMeta, State};

fn initial_state_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("initial_state.cbor")
}

fn actions_path<P: AsRef<Path>>(path: P, file_index: usize) -> PathBuf {
    path.as_ref().join(format!("actions_{}.cbor", file_index))
}

#[derive(Serialize, Deserialize)]
pub struct RecordedInitialState<'a> {
    pub rng_seed: u64,
    pub state: Cow<'a, State>,
}

impl<'a> RecordedInitialState<'a> {
    pub fn write_to<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), ciborium::ser::Error<std::io::Error>> {
        ciborium::ser::into_writer(self, writer)?;
        Ok(())
    }

    pub fn decode(encoded: &[u8]) -> Result<Self, ciborium::de::Error<std::io::Error>>
    where
        Self: serde::de::DeserializeOwned,
    {
        let mut cursor = std::io::Cursor::new(encoded);
        let decoded = ciborium::de::from_reader(&mut cursor)?;
        Ok(decoded)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordedActionWithMeta<'a> {
    pub kind: ActionKind,
    pub meta: redux::ActionMeta,
    pub action: Option<Cow<'a, Action>>,
}

impl<'a> RecordedActionWithMeta<'a> {
    pub fn encode(&self) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
        let mut buffer = Vec::new();
        ciborium::ser::into_writer(self, &mut buffer)?;
        Ok(buffer)
    }

    pub fn decode(encoded: &[u8]) -> Result<Self, ciborium::de::Error<std::io::Error>>
    where
        Self: serde::de::DeserializeOwned,
    {
        let mut cursor = std::io::Cursor::new(encoded);
        let decoded = ciborium::de::from_reader(&mut cursor)?;
        Ok(decoded)
    }

    pub fn as_action_with_meta(self) -> Result<ActionWithMeta, Self> {
        if self.action.is_some() {
            let action = self.action.unwrap().into_owned();
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
