//! Defines the service interface for archiving applied blocks,
//! allowing historical blockchain data to be stored persistently.

use crate::ledger::write::BlockApplyResult;

pub trait ArchiveService: redux::Service {
    fn send_to_archive(&mut self, data: BlockApplyResult);
}
