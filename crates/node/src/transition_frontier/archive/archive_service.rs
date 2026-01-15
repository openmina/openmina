use crate::ledger::write::BlockApplyResult;

pub trait ArchiveService: redux::Service {
    fn send_to_archive(&mut self, data: BlockApplyResult);
}
