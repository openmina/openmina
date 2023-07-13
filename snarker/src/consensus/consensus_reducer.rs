use crate::consensus::{
    is_short_range_fork, long_range_fork_take, short_range_fork_take, ConsensusAction,
    ConsensusActionWithMetaRef, ConsensusBlockState, ConsensusBlockStatus,
    ConsensusLongRangeForkDecision, ConsensusShortRangeForkDecision, ConsensusState,
};

impl ConsensusState {
    pub fn reducer(&mut self, action: ConsensusActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            ConsensusAction::BlockReceived(a) => {
                self.blocks.insert(
                    a.hash.clone(),
                    ConsensusBlockState {
                        block: a.block.clone(),
                        status: ConsensusBlockStatus::Received { time: meta.time() },
                        history: a.history.clone(),
                    },
                );
            }
            ConsensusAction::BlockSnarkVerifyPending(a) => {
                if let Some(block) = self.blocks.get_mut(&a.hash) {
                    block.status = ConsensusBlockStatus::SnarkVerifyPending {
                        time: meta.time(),
                        req_id: a.req_id,
                    };
                }
            }
            ConsensusAction::BlockSnarkVerifySuccess(a) => {
                if let Some(block) = self.blocks.get_mut(&a.hash) {
                    block.status = ConsensusBlockStatus::SnarkVerifySuccess { time: meta.time() };
                }
            }
            ConsensusAction::DetectForkRange(a) => {
                let candidate_hash = &a.hash;
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
                    shared::log::debug!(shared::log::system_time(); kind = "ConsensusAction::DetectForkRange", status = serde_json::to_string(&candidate_state.status).unwrap());
                }
                shared::log::debug!(shared::log::system_time(); kind = "ConsensusAction::DetectForkRange");
            }
            ConsensusAction::ShortRangeForkResolve(a) => {
                let candidate_hash = &a.hash;
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
                                    &tip.hash,
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
                            candidate.history.take();
                        }

                        candidate.status = ConsensusBlockStatus::ShortRangeForkResolve {
                            time: meta.time(),
                            compared_with: best_tip_hash,
                            decision,
                        };
                    }
                }
            }
            ConsensusAction::LongRangeForkResolve(a) => {
                shared::log::debug!(shared::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve");
                let candidate_hash = &a.hash;
                let Some(tip_ref) = self.best_tip() else {
                    return;
                };
                let Some(candidate_state) = self.blocks.get(candidate_hash) else {
                    return;
                };
                shared::log::debug!(shared::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve", pre_status = serde_json::to_string(&candidate_state.status).unwrap());
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
                        candidate_state.history = None;
                        ConsensusLongRangeForkDecision::Keep(why)
                    },
                };
                shared::log::debug!(shared::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve", status = serde_json::to_string(&candidate_state.status).unwrap());
            }
            ConsensusAction::BestTipUpdate(a) => {
                self.best_tip = Some(a.hash.clone());

                if let Some(tip) = self.blocks.get_mut(&a.hash) {
                    let pred_level = match tip.height().checked_sub(1) {
                        Some(v) => v as u32,
                        None => return,
                    };
                    if let Some(history) = tip.history.take() {
                        self.update_best_tip_history(pred_level, &history);
                    } else {
                        let pred_hash = tip.block.header.protocol_state.previous_state_hash.clone();

                        if self
                            .is_part_of_main_chain(pred_level, &pred_hash)
                            .unwrap_or(false)
                        {
                            self.update_best_tip_history(pred_level, &[pred_hash]);
                            return;
                        }
                    }
                }
            }
            ConsensusAction::BestTipHistoryUpdate(a) => {
                if let Some(tip) = self.blocks.get_mut(&a.tip_hash) {
                    let pred_level = match tip.height().checked_sub(1) {
                        Some(v) => v as u32,
                        None => return,
                    };
                    self.update_best_tip_history(pred_level, &a.history);
                }
            }
        }
    }
}
