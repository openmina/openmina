use std::borrow::Cow;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::{Action, ActionWithMeta, EventSourceAction, State};

use super::{RecordedActionWithMeta, RecordedInitialState};

pub enum Recorder {
    None,
    OnlyInputActions {
        recorder_path: PathBuf,
        actions_f: fs::File,
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

        Self::OnlyInputActions {
            recorder_path: path,
            actions_f: fs::File::create(actions_path)
                .expect("creating file for openmina recorder initial state failed!"),
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
                actions_f,
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

                let file = if *actions_f_bytes_written > 64 * 1024 * 1024 {
                    actions_f.sync_all().unwrap();
                    *actions_f_bytes_written = 0;
                    *actions_f_index += 1;
                    *actions_f =
                        fs::File::create(super::actions_path(recorder_path, *actions_f_index))
                            .unwrap();
                    actions_f
                } else {
                    actions_f
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
}

impl Drop for Recorder {
    fn drop(&mut self) {
        match self {
            Self::None => {}
            Self::OnlyInputActions { actions_f, .. } => {
                eprintln!("Flushing recorded actions to disk before shutdown");
                let _ = actions_f.sync_all();
            }
        }
    }
}
