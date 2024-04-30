use mina_p2p_messages::v2::LedgerHash;

pub trait TransitionFrontierSyncLedgerStagedService: redux::Service {
    // TODO(tizoc): Only used for the current workaround to make staged ledger
    // reconstruction async, can be removed when the ledger services are made async
    fn staged_ledger_reconstruct_result_store(&self, staged_ledger_hash: LedgerHash);
}
