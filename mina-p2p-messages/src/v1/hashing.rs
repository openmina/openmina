use mina_hasher::{Hashable, Hasher, ROInput};

use super::generated;
use super::state_hash::{StateHashStable, StateHashStableV1};
use crate::versioned::{Ver, Versioned};

impl generated::MinaStateProtocolStateValueStableV1Versioned {
    pub fn hash(&self, hasher: &mut impl Hasher<Self>) -> StateHashStable {
        let field = hasher.hash(self);
        StateHashStableV1::from_bigint(field.into()).into()
    }
}

impl generated::MinaStateProtocolStateValueStableV1VersionedV1 {
    pub fn hash(&self, hasher: &mut impl Hasher<Self>) -> StateHashStable {
        let field = hasher.hash(self);
        StateHashStableV1::from_bigint(field.into()).into()
    }
}

impl<T, const V: Ver> Hashable for Versioned<T, V>
where
    T: Hashable,
{
    type D = T::D;

    fn to_roinput(&self) -> mina_hasher::ROInput {
        self.inner().to_roinput()
    }

    fn domain_string(domain_param: Self::D) -> Option<String> {
        T::domain_string(domain_param)
    }
}

impl Hashable for generated::UnsignedExtendedUInt32StableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        // TODO: should be u32 not i32?
        ROInput::new().append_u32(self.0 .0 as u32)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::UnsignedExtendedUInt64StableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        // TODO: should be u64 not i64?
        ROInput::new().append_u64(self.0 .0 as u64)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for StateHashStableV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new().append_hashable(&self.0 .0)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::MinaStateProtocolStateValueStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> mina_hasher::ROInput {
        let data = self.0.inner();

        // TODO: hasher initialization is costly. Here we have to hash
        // `protocol_state.body` before we append it as a roinput, but
        // Hashable api isn't flexible enough to allow us to do that,
        // without initializing hasher here. This needs to be fixed/improved.
        let mut hasher = mina_hasher::create_legacy(());
        let body_hash = hasher.hash(&data.body);
        ROInput::new()
            .append_hashable(&data.previous_state_hash)
            .append_field(body_hash)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        Some("CodaProtoState".into())
    }
}

impl Hashable for generated::MinaStateProtocolStateBodyValueStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        ROInput::new()
            .append_hashable(&data.constants)
            .append_hashable(&data.genesis_state_hash)
            .append_hashable(&data.blockchain_state)
            .append_hashable(&data.consensus_state)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        // None
        Some("CodaProtoStateBody".into())
    }
}

impl Hashable for generated::MinaBaseProtocolConstantsCheckedValueStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        ROInput::new()
            .append_hashable(&data.k)
            .append_hashable(&data.delta)
            .append_hashable(&data.slots_per_epoch)
            .append_hashable(&data.slots_per_sub_window)
            .append_hashable(&data.genesis_state_timestamp)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable
    for generated::ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0V1
{
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new().append_hashable(self.0.inner())
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::BlockTimeTimeStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new().append_hashable(self.0.inner())
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::MinaStateBlockchainStateValueStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        ROInput::new()
            .append_hashable(&data.staged_ledger_hash)
            .append_hashable(&data.snarked_ledger_hash)
            .append_hashable(&data.genesis_ledger_hash)
            .append_hashable(&data.snarked_next_available_token)
            .append_hashable(&data.timestamp)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::MinaBaseStagedLedgerHashStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        ROInput::new()
            .append_hashable(&data.non_snark)
            .append_hashable(&data.pending_coinbase_hash.inner().0.inner().0 .0)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl generated::MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1 {
    pub fn sha256(&self) -> Vec<u8> {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        let ledger_hash_bytes: Vec<u8> = self.ledger_hash.inner().0 .0.iter_bytes().rev().collect();
        hasher.update(&ledger_hash_bytes);
        hasher.update(self.aux_hash.inner().0.as_ref());
        hasher.update(self.pending_coinbase_aux.inner().0.as_ref());
        hasher.finalize().to_vec()
    }
}

impl Hashable for generated::MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new().append_bytes(&self.sha256())
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::MinaBaseTokenIdStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        // TODO: should be u64 not i64?
        ROInput::new().append_u64(self.0.inner().0.inner().0 .0 as u64)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        let mut roi = ROInput::new()
            .append_hashable(&data.blockchain_length)
            .append_hashable(&data.epoch_count)
            .append_hashable(&data.min_window_density);
        for v in &data.sub_window_densities {
            roi = roi.append_hashable(v);
        }
        roi.append_hashable(&data.last_vrf_output)
            .append_hashable(&data.total_currency)
            .append_hashable(&data.curr_global_slot)
            .append_hashable(&data.global_slot_since_genesis)
            .append_bool(data.has_ancestor_in_same_checkpoint_window)
            .append_bool(data.supercharge_coinbase)
            .append_hashable(&data.staking_epoch_data)
            .append_hashable(&data.next_epoch_data)
            .append_hashable(&data.block_stake_winner.inner().0 .0.inner().x)
            .append_bool(data.block_stake_winner.inner().0 .0.inner().is_odd)
            .append_hashable(&data.block_creator.inner().0 .0.inner().x)
            .append_bool(data.block_creator.inner().0 .0.inner().is_odd)
            .append_hashable(&data.coinbase_receiver.inner().0 .0.inner().x)
            .append_bool(data.coinbase_receiver.inner().0 .0.inner().is_odd)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::ConsensusVrfOutputTruncatedStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.as_ref();
        let roi = ROInput::new();
        if data.len() <= 31 {
            roi.append_bytes(data)
        } else {
            let roi = roi.append_bytes(&data[..31]);
            if data.len() > 31 {
                let last = data[31];
                roi.append_bool(last & 0b1 > 0)
                    .append_bool(last & 0b10 > 0)
                    .append_bool(last & 0b100 > 0)
                    .append_bool(last & 0b1000 > 0)
                    .append_bool(last & 0b10000 > 0)
            } else {
                roi
            }
        }
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::CurrencyAmountMakeStrStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new().append_hashable(self.0.inner())
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::ConsensusGlobalSlotStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        ROInput::new()
            .append_hashable(&data.slot_number)
            .append_hashable(&data.slots_per_epoch)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::ConsensusGlobalSlotStableV1VersionedV1PolyArg0V1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new().append_hashable(self.0.inner())
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable
    for generated::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1
{
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        ROInput::new()
            .append_hashable(&data.seed)
            .append_hashable(&data.start_checkpoint)
            .append_hashable(&data.epoch_length)
            .append_hashable(&data.ledger)
            .append_hashable(&data.lock_checkpoint)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1V1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new().append_hashable(&self.0 .0)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::MinaBaseEpochLedgerValueStableV1VersionedV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        ROInput::new()
            .append_hashable(&data.hash)
            .append_hashable(&data.total_currency)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable for generated::MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHashV1 {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new().append_hashable(&self.0 .0)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}

impl Hashable
    for generated::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1VersionedV1
{
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let data = self.0.inner();
        ROInput::new()
            .append_hashable(&data.seed)
            .append_hashable(&data.start_checkpoint)
            .append_hashable(&data.epoch_length)
            .append_hashable(&data.ledger)
            .append_hashable(&data.lock_checkpoint)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
    }
}
