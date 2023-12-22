use crate::core::snark::SnarkJobId;

use super::JobState;

pub trait SnarkPoolService: redux::Service {
    fn random_choose<'a>(
        &mut self,
        iter: impl Iterator<Item = &'a JobState>,
        n: usize,
    ) -> Vec<SnarkJobId>;
}
