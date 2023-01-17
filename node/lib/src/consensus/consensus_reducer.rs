use super::{
    ConsensusAction, ConsensusActionWithMetaRef, ConsensusBlockState, ConsensusBlockStatus,
    ConsensusShortRangeForkDecision, ConsensusShortRangeForkDecisionIgnoreReason,
    ConsensusShortRangeForkDecisionUseReason, ConsensusState,
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
            ConsensusAction::ShortRangeForkResolve(a) => {
                let candidate_hash = &a.hash;
                if let Some(candidate) = self.blocks.get(candidate_hash) {
                    let (best_tip_hash, decision): (_, ConsensusShortRangeForkDecision) = match self
                        .best_tip()
                    {
                        Some(tip) => (Some(tip.hash.clone()), {
                            let tip_cs = &tip.header.protocol_state.body.consensus_state;
                            let candidate_cs =
                                &candidate.block.header.protocol_state.body.consensus_state;
                            if tip_cs.blockchain_length.0 < candidate_cs.blockchain_length.0 {
                                ConsensusShortRangeForkDecisionUseReason::LongerChain.into()
                            } else if tip_cs.blockchain_length.0 == candidate_cs.blockchain_length.0
                            {
                                let tip_vrf = tip_cs.last_vrf_output.blake2b();
                                let candidate_vrf = candidate_cs.last_vrf_output.blake2b();

                                match candidate_vrf.cmp(&tip_vrf) {
                                    std::cmp::Ordering::Greater => {
                                        ConsensusShortRangeForkDecisionUseReason::BiggerVrf.into()
                                    }
                                    std::cmp::Ordering::Less => {
                                        ConsensusShortRangeForkDecisionIgnoreReason::SmallerVrf
                                            .into()
                                    }
                                    std::cmp::Ordering::Equal => {
                                        if candidate_hash > &tip.hash {
                                            ConsensusShortRangeForkDecisionUseReason::TieBreakerBiggerStateHash.into()
                                        } else {
                                            ConsensusShortRangeForkDecisionIgnoreReason::TieBreakerSmallerStateHash.into()
                                        }
                                    }
                                }
                            } else {
                                ConsensusShortRangeForkDecisionIgnoreReason::ShorterChain.into()
                            }
                        }),
                        None => (
                            None,
                            ConsensusShortRangeForkDecisionUseReason::NoBestTip.into(),
                        ),
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
