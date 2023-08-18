use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::{RecordedActionWithMeta, RecordedInitialState};

pub struct StateWithInputActionsReader {
    dir: PathBuf,
}

impl StateWithInputActionsReader {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
        }
    }

    pub fn initial_state_path(&self) -> PathBuf {
        super::initial_state_path(&self.dir)
    }

    pub fn read_initial_state(&self) -> Result<RecordedInitialState, Box<dyn Error>> {
        let path = self.initial_state_path();
        let encoded = fs::read(path)?;
        Ok(RecordedInitialState::decode(&encoded)?)
    }

    pub fn read_actions(
        &self,
    ) -> impl Iterator<Item = (PathBuf, impl Iterator<Item = RecordedActionWithMeta<'_>>)> {
        (1..).map_while(move |file_index| {
            let path = super::actions_path(&self.dir, file_index);
            let mut file = fs::File::open(&path).ok()?;

            let iter = std::iter::repeat(()).map_while(move |_| {
                let mut len_bytes = [0; 8];
                file.read_exact(&mut len_bytes).ok()?;
                let len = u64::from_be_bytes(len_bytes);

                let mut data = vec![0; len as usize];
                file.read_exact(&mut data).unwrap();
                Some(RecordedActionWithMeta::decode(&data).unwrap())
            });
            Some((path, iter))
        })
    }
}
