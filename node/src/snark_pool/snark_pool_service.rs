use crate::core::snark::SnarkJobId;

pub trait SnarkPoolService: redux::Service {
    fn random_choose<'a>(
        &mut self,
        iter: impl Iterator<Item = &'a SnarkJobId>,
        n: usize,
    ) -> Vec<SnarkJobId>;
}
