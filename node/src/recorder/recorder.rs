use std::borrow::Cow;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, TryLockError};

use crate::p2p::identity::SecretKey as P2pSecretKey;
use crate::{Action, ActionWithMeta, EventSourceAction, State};

use super::{RecordedActionWithMeta, RecordedInitialState};

static ACTIONS_F: Mutex<Vec<Option<fs::File>>> = Mutex::new(Vec::new());

/// Panics: if all the `Recorder` instances aren't in the same thread.
pub enum Recorder {
    None,
    OnlyInputActions {
        recorder_i: usize,
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
        let mut actions_files = ACTIONS_F.try_lock().unwrap();
        actions_files.push(Some(file));

        Self::OnlyInputActions {
            recorder_i: actions_files.len() - 1,
            recorder_path: path,
            actions_f_bytes_written: 0,
            actions_f_index,
        }
    }

    pub fn initial_state(&mut self, rng_seed: [u8; 32], p2p_sec_key: P2pSecretKey, state: &State) {
        match self {
            Self::None => {}
            Self::OnlyInputActions { recorder_path, .. } => {
                let initial_state = RecordedInitialState {
                    rng_seed,
                    p2p_sec_key,
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
                recorder_i,
                recorder_path,
                actions_f_bytes_written,
                actions_f_index,
                ..
            } => {
                let is_input = match action.action() {
                    Action::CheckTimeouts(_) => true,
                    Action::EventSource(e) => match e {
                        EventSourceAction::NewEvent { .. } => true,
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

                let mut files = ACTIONS_F.try_lock().unwrap();
                let cur_f = &mut files[*recorder_i];

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
        graceful_shutdown(None)
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        match self {
            Self::None => {}
            Self::OnlyInputActions { recorder_i, .. } => graceful_shutdown(Some(*recorder_i)),
        }
    }
}

fn graceful_shutdown(only_i: Option<usize>) {
    let Some(mut files) = ACTIONS_F.try_lock().map_or_else(
        |err| match err {
            TryLockError::WouldBlock => None,
            TryLockError::Poisoned(v) => Some(v.into_inner()),
        },
        Some,
    ) else {
        return;
    };
    let files_iter = files
        .iter_mut()
        .enumerate()
        .filter(|(i, _)| only_i.map_or(true, |only_i| only_i == *i))
        .filter_map(|(i, v)| Some((i, v.take()?)));

    for (i, file) in files_iter {
        eprintln!("Flushing recorded actions to disk before shutdown. i={i}");
        let _ = file.sync_all();
    }
}
