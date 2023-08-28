use std::borrow::Cow;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, TryLockError};

use crate::{Action, ActionWithMeta, EventSourceAction, State};

use super::{RecordedActionWithMeta, RecordedInitialState};

static ACTIONS_F: Mutex<Option<fs::File>> = Mutex::new(None);

/// There must only be 1 `Recorder` instance per process!
pub enum Recorder {
    None,
    OnlyInputActions {
        recorder_path: PathBuf,
        actions_f_bytes_written: u64,
        actions_f_index: usize,
    },
}

impl Recorder {
    pub fn only_input_actions<P: AsRef<Path>>(work_dir: P) -> Self {
        let path = work_dir.as_ref().join("recorder");

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("creating dir for openmina recorder failed!");

        let actions_f_index = 1;
        let actions_path = super::actions_path(&path, actions_f_index);

        let file = fs::File::create(actions_path)
            .expect("creating file for openmina recorder initial state failed!");
        let _ = ACTIONS_F.try_lock().unwrap().insert(file);

        Self::OnlyInputActions {
            recorder_path: path,
            actions_f_bytes_written: 0,
            actions_f_index,
        }
    }

    pub fn initial_state(&mut self, rng_seed: u64, state: &State) {
        match self {
            Self::None => {}
            Self::OnlyInputActions { recorder_path, .. } => {
                let initial_state = RecordedInitialState {
                    rng_seed,
                    state: Cow::Borrowed(state),
                };
                let initial_state_path = super::initial_state_path(recorder_path);
                let mut initial_state_f = fs::File::create(initial_state_path)
                    .expect("creating file for openmina recorder initial state failed!");
                initial_state.write_to(&mut initial_state_f).unwrap();
                initial_state_f.sync_all().unwrap();
            }
        }
    }

    pub fn action(&mut self, action: &ActionWithMeta) {
        match self {
            Self::None => {}
            Self::OnlyInputActions {
                recorder_path,
                actions_f_bytes_written,
                actions_f_index,
                ..
            } => {
                let is_input = match action.action() {
                    Action::CheckTimeouts(_) => true,
                    Action::EventSource(e) => match e {
                        EventSourceAction::NewEvent(_) => true,
                        _ => return,
                    },
                    _ => false,
                };

                let data = if !is_input {
                    let kind = action.action().kind();
                    RecordedActionWithMeta::from((kind, action.meta().clone()))
                } else {
                    RecordedActionWithMeta::from(action)
                };

                let mut cur_f = ACTIONS_F.try_lock().unwrap();

                let file = if *actions_f_bytes_written > 64 * 1024 * 1024 {
                    cur_f.take().unwrap().sync_all().unwrap();
                    *actions_f_bytes_written = 0;
                    *actions_f_index += 1;
                    cur_f.insert(
                        fs::File::create(super::actions_path(recorder_path, *actions_f_index))
                            .unwrap(),
                    )
                } else {
                    cur_f.as_mut().unwrap()
                };

                let mut writer = BufWriter::new(file);

                let encoded = data.encode().unwrap();
                writer
                    .write_all(&(encoded.len() as u64).to_be_bytes())
                    .unwrap();
                writer.write_all(&encoded).unwrap();
                writer.flush().unwrap();

                *actions_f_bytes_written += 8 + encoded.len() as u64;
            }
        }
    }

    pub fn graceful_shutdown() {
        graceful_shutdown()
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        match self {
            Self::None => {}
            Self::OnlyInputActions { .. } => {
                graceful_shutdown();
            }
        }
    }
}

pub fn graceful_shutdown() {
    let Some(f) = ACTIONS_F
        .try_lock()
        .map(|mut v| v.take())
        .unwrap_or_else(|err| match err {
            TryLockError::WouldBlock => None,
            TryLockError::Poisoned(v) => v.into_inner().take(),
        })
    else {
        return;
    };

    eprintln!("Flushing recorded actions to disk before shutdown");
    let _ = f.sync_all();
}
