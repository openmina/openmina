use openmina_core::consensus::{is_short_range_fork, long_range_fork_take, short_range_fork_take};

use super::{
    ConsensusAction, ConsensusActionWithMetaRef, ConsensusBlockState, ConsensusBlockStatus,
    ConsensusLongRangeForkDecision, ConsensusShortRangeForkDecision, ConsensusState,
};

impl ConsensusState {
    pub fn reducer(&mut self, action: ConsensusActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            ConsensusAction::BlockReceived {
                hash,
                block,
                chain_proof,
            } => {
                self.blocks.insert(
                    hash.clone(),
                    ConsensusBlockState {
                        block: block.clone(),
                        status: ConsensusBlockStatus::Received { time: meta.time() },
                        chain_proof: chain_proof.clone(),
                    },
                );
            }
            ConsensusAction::BlockChainProofUpdate { hash, chain_proof } => {
                if self.best_tip.as_ref() == Some(hash) {
                    self.best_tip_chain_proof = Some(chain_proof.clone());
                } else if let Some(block) = self.blocks.get_mut(hash) {
                    block.chain_proof = Some(chain_proof.clone());
                }
            }
            ConsensusAction::BlockSnarkVerifyPending { req_id, hash } => {
                if let Some(block) = self.blocks.get_mut(hash) {
                    block.status = ConsensusBlockStatus::SnarkVerifyPending {
                        time: meta.time(),
                        req_id: *req_id,
                    };
                }
            }
            ConsensusAction::BlockSnarkVerifySuccess { hash } => {
                if let Some(block) = self.blocks.get_mut(hash) {
                    block.status = ConsensusBlockStatus::SnarkVerifySuccess { time: meta.time() };
                }
            }
            ConsensusAction::DetectForkRange { hash } => {
                let candidate_hash = hash;
                let Some(candidate_state) = self.blocks.get(candidate_hash) else {
                    return;
                };
                let candidate = &candidate_state.block.header;
                let (tip_hash, short_fork) = if let Some(tip_ref) = self.best_tip() {
                    let tip = tip_ref.header;
                    (
                        Some(tip_ref.hash.clone()),
                        is_short_range_fork(
                            &candidate.protocol_state.body.consensus_state,
                            &tip.protocol_state.body.consensus_state,
                        ),
                    )
                } else {
                    (None, true)
                };
                if let Some(candidate_state) = self.blocks.get_mut(candidate_hash) {
                    candidate_state.status = ConsensusBlockStatus::ForkRangeDetected {
                        time: meta.time(),
                        compared_with: tip_hash,
                        short_fork,
                    };
                    openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::DetectForkRange", status = serde_json::to_string(&candidate_state.status).unwrap());
                }
                openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::DetectForkRange");
            }
            ConsensusAction::ShortRangeForkResolve { hash } => {
                let candidate_hash = hash;
                if let Some(candidate) = self.blocks.get(candidate_hash) {
                    let (best_tip_hash, decision): (_, ConsensusShortRangeForkDecision) =
                        match self.best_tip() {
                            Some(tip) => (Some(tip.hash.clone()), {
                                let tip_cs = &tip.header.protocol_state.body.consensus_state;
                                let candidate_cs =
                                    &candidate.block.header.protocol_state.body.consensus_state;
                                let (take, why) = short_range_fork_take(
                                    tip_cs,
                                    candidate_cs,
tip.hash,
                                    candidate_hash,
                                );
                                if take {
                                    ConsensusShortRangeForkDecision::Take(why)
                                } else {
                                    ConsensusShortRangeForkDecision::Keep(why)
                                }
                            }),
                            None => (None, ConsensusShortRangeForkDecision::TakeNoBestTip),
                        };

                    if let Some(candidate) = self.blocks.get_mut(candidate_hash) {
                        if !decision.use_as_best_tip() {
                            candidate.chain_proof = None;
                        }

                        candidate.status = ConsensusBlockStatus::ShortRangeForkResolve {
                            time: meta.time(),
                            compared_with: best_tip_hash,
                            decision,
                        };
                    }
                }
            }
            ConsensusAction::LongRangeForkResolve { hash } => {
                openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve");
                let candidate_hash = hash;
                let Some(tip_ref) = self.best_tip() else {
                    return;
                };
                let Some(candidate_state) = self.blocks.get(candidate_hash) else {
                    return;
                };
                openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve", pre_status = serde_json::to_string(&candidate_state.status).unwrap());
                let tip_hash = tip_ref.hash.clone();
                let tip = tip_ref.header;
                let tip_cs = &tip.protocol_state.body.consensus_state;
                let candidate = &candidate_state.block.header;
                let candidate_cs = &candidate.protocol_state.body.consensus_state;

                let (take, why) =
                    long_range_fork_take(tip_cs, candidate_cs, &tip_hash, candidate_hash);

                let Some(candidate_state) = self.blocks.get_mut(candidate_hash) else {
                    return;
                };
                candidate_state.status = ConsensusBlockStatus::LongRangeForkResolve {
                    time: meta.time(),
                    compared_with: tip_hash,
                    decision: if take {
                        ConsensusLongRangeForkDecision::Take(why)
                    } else {
                        candidate_state.chain_proof = None;
                        ConsensusLongRangeForkDecision::Keep(why)
                    },
                };
                openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve", status = serde_json::to_string(&candidate_state.status).unwrap());
            }
            ConsensusAction::BestTipUpdate { hash } => {
                self.best_tip = Some(hash.clone());

                if let Some(tip) = self.blocks.get_mut(hash) {
                    self.best_tip_chain_proof = tip.chain_proof.take();
                }
            }
            ConsensusAction::Prune => {
                let Some(best_tip_hash) = self.best_tip.clone() else {
                    return;
                };
                let blocks = &mut self.blocks;

                // keep at most latest 32 candidate blocks.
                let blocks_to_keep = (0..32)
                    .scan(best_tip_hash, |block_hash, _| {
                        let block_state = blocks.remove(block_hash) ?;
                        let block_hash = match block_state.status.compared_with() {
                            None => block_hash.clone(),
                            Some(compared_with) => {
                                std::mem::replace(block_hash, compared_with.clone())
                            }
                        };
                        Some((block_hash, block_state))
                    })
                    .collect();
                *blocks = blocks_to_keep;
            }
        }
    }
}
