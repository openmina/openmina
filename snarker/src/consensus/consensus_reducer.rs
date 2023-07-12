use crate::consensus::{is_short_range_fork, long_range_fork_take, ConsensusLongRangeForkDecision};

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

#[cfg(test)]
mod tests {
    use mina_p2p_messages::v2::{MinaStateProtocolStateValueStableV2, StateHash};

    use super::long_range_fork_take;

    fn long_range_fork_test(
        tip: &str,
        cnd: &str,
        tip_hash: &str,
        cnd_hash: &str,
        expect_take: bool,
    ) {
        let tip = serde_json::from_str::<MinaStateProtocolStateValueStableV2>(tip).unwrap();
        let cnd = serde_json::from_str::<MinaStateProtocolStateValueStableV2>(cnd).unwrap();
        let tip_hash = tip_hash.parse::<StateHash>().unwrap();
        let cnd_hash = cnd_hash.parse::<StateHash>().unwrap();

        let (take, _) = long_range_fork_take(
            &tip.body.consensus_state,
            &cnd.body.consensus_state,
            &tip_hash,
            &cnd_hash,
        );
        assert_eq!(take, expect_take);
    }

    macro_rules! long_fork_test {
        ($prefix:expr, $tip:expr, $cnd:expr, $decision:expr) => {
            let tip_str = include_str!(concat!(
                "../../../tests/files/forks/long-",
                $prefix,
                "-",
                $tip,
                "-",
                $cnd,
                "-tip.json"
            ));
            let cnd_str = include_str!(concat!(
                "../../../tests/files/forks/long-",
                $prefix,
                "-",
                $tip,
                "-",
                $cnd,
                "-cnd.json"
            ));
            long_range_fork_test(tip_str, cnd_str, $tip, $cnd, $decision);
        };

        (take $prefix:expr, $tip:expr, $cnd:expr) => {
            long_fork_test!(concat!("take-", $prefix), $tip, $cnd, true);
        };

        (keep $prefix:expr, $tip:expr, $cnd:expr) => {
            long_fork_test!(concat!("keep-", $prefix), $tip, $cnd, false);
        };
    }

    #[test]
    fn long_range_fork() {
        long_fork_test!(
            take
                "density-92-97",
            "3NLESd9gzU52bDWSXL5uUAYbCojHXSVdeBX4sCMF3V8Ns9D1Sriy",
            "3NLQfKJ4kBagLgmiwyiVw9zbi53tiNy8TNu2ua1jmCyEecgbBJoN"
        );
        long_fork_test!(
            keep
                "density-161-166",
            "3NKY1kxHMRfjBbjfAA5fsasUCWFF9B7YqYFfNH4JFku6ZCUUXyLG",
            "3NLFoBQ6y3nku79LQqPgKBmuo5Ngnpr7rfZygzdRrcPtz2gewRFC"
        );
    }
}
