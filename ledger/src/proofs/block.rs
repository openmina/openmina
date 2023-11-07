use std::{path::Path, str::FromStr, sync::Arc};

use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    dummy,
    proofs::witness::Boolean,
    scan_state::{
        protocol_state::MinaHash,
        scan_state::transaction_snark::{SokDigest, Statement},
    },
    Inputs, ToInputs,
};

use super::{
    to_field_elements::ToFieldElements,
    witness::{checked_hash2, transaction_snark::checked_hash, Check, Prover, Witness},
};

fn read_witnesses() -> Vec<Fp> {
    let f = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("rampup4")
            .join("block_fps.txt"),
    )
    .unwrap();

    let fps = f
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| Fp::from_str(s).unwrap())
        .collect::<Vec<_>>();

    fps
}

impl ToFieldElements<Fp> for v2::MinaStateSnarkTransitionValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            blockchain_state,
            consensus_transition,
            pending_coinbase_update:
                v2::MinaBasePendingCoinbaseUpdateStableV1 {
                    action,
                    coinbase_amount,
                },
        } = self;

        blockchain_state.to_field_elements(fields);
        fields.push(Fp::from(consensus_transition.as_u32()));

        // https://github.com/MinaProtocol/mina/blob/f6fb903bef974b191776f393a2f9a1e6360750fe/src/lib/mina_base/pending_coinbase.ml#L420
        use v2::MinaBasePendingCoinbaseUpdateActionStableV1::*;
        let bits = match action {
            UpdateNone => [Boolean::False, Boolean::False],
            UpdateOne => [Boolean::True, Boolean::False],
            UpdateTwoCoinbaseInFirst => [Boolean::False, Boolean::True],
            UpdateTwoCoinbaseInSecond => [Boolean::True, Boolean::True],
        };
        fields.extend(bits.into_iter().map(Boolean::to_field::<Fp>));
        fields.push(Fp::from(coinbase_amount.as_u64()));
    }
}

impl Check<Fp> for v2::MinaStateSnarkTransitionValueStableV2 {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            blockchain_state,
            consensus_transition,
            pending_coinbase_update:
                v2::MinaBasePendingCoinbaseUpdateStableV1 {
                    action: _,
                    coinbase_amount,
                },
        } = self;

        blockchain_state.check(w);
        consensus_transition.check(w);
        coinbase_amount.check(w);
    }
}

impl ToFieldElements<Fp> for v2::MinaStateProtocolStateValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            previous_state_hash,
            body,
        } = self;

        previous_state_hash
            .to_field::<Fp>()
            .to_field_elements(fields);
        body.to_field_elements(fields);
    }
}

impl Check<Fp> for v2::MinaStateProtocolStateValueStableV2 {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            previous_state_hash: _,
            body,
        } = self;

        body.check(w);
    }
}

fn ledger_proof_opt(
    proof: Option<&v2::LedgerProofProdStableV2>,
    next_state: &v2::MinaStateProtocolStateValueStableV2,
) -> (Statement<SokDigest>, Arc<v2::TransactionSnarkProofStableV2>) {
    match proof {
        Some(proof) => {
            let statement: Statement<SokDigest> = (&proof.0.statement).into();
            let p: &v2::TransactionSnarkProofStableV2 = &proof.0.proof;
            // TODO: Don't clone the proof here
            (statement, Arc::new(p.clone()))
        }
        None => {
            let statement: Statement<()> =
                (&next_state.body.blockchain_state.ledger_proof_statement).into();
            let statement = statement.with_digest(SokDigest::default());
            let p = dummy::dummy_transaction_proof();
            (statement, p)
        }
    }
}

fn checked_hash_protocol_state(
    state: &v2::MinaStateProtocolStateValueStableV2,
    w: &mut Witness<Fp>,
) -> (Fp, Fp) {
    let v2::MinaStateProtocolStateValueStableV2 {
        previous_state_hash,
        body,
    } = state;

    let mut inputs = Inputs::new();
    body.to_inputs(&mut inputs);
    let body_hash = checked_hash("MinaProtoStateBody", &inputs.to_fields(), w);

    let mut inputs = Inputs::new();
    inputs.append_field(previous_state_hash.to_field());
    inputs.append_field(body_hash);
    let hash = checked_hash("MinaProtoState", &inputs.to_fields(), w);

    (hash, body_hash)
}

pub struct ProverExtendBlockchainInputStableV22 {
    pub chain: v2::BlockchainSnarkBlockchainStableV2,
    pub next_state: v2::MinaStateProtocolStateValueStableV2,
    pub block: v2::MinaStateSnarkTransitionValueStableV2,
    pub ledger_proof: Option<v2::LedgerProofProdStableV2>,
    pub prover_state: v2::ConsensusStakeProofStableV2,
    pub pending_coinbase: v2::MinaBasePendingCoinbaseWitnessStableV2,
}

struct BlockProofParams<'a> {
    transition: &'a v2::MinaStateSnarkTransitionValueStableV2,
    prev_state: &'a v2::MinaStateProtocolStateValueStableV2,
    prev_state_proof: &'a v2::MinaBaseProofStableV2,
    txn_snark: &'a Statement<SokDigest>,
    txn_snark_proof: &'a v2::TransactionSnarkProofStableV2,
}

pub fn generate_block_proof(
    input: &v2::ProverExtendBlockchainInputStableV2,
    block_prover: &Prover<Fp>,
    wrap_prover: &Prover<Fq>,
    w: &mut Witness<Fp>,
) {
    w.ocaml_aux = read_witnesses();

    let v2::ProverExtendBlockchainInputStableV2 {
        chain,
        next_state,
        block,
        ledger_proof,
        prover_state,
        pending_coinbase,
    } = input;

    let (txn_snark_statement, txn_snark_proof) =
        ledger_proof_opt(ledger_proof.as_ref(), next_state);

    let params = BlockProofParams {
        transition: block,
        prev_state: &chain.state,
        prev_state_proof: &chain.proof,
        txn_snark: &txn_snark_statement,
        txn_snark_proof: &txn_snark_proof,
    };

    let BlockProofParams {
        transition,
        prev_state,
        prev_state_proof,
        txn_snark,
        txn_snark_proof,
    } = params;

    let new_state_hash = w.exists(MinaHash::hash(next_state));
    w.exists(transition);
    w.exists(txn_snark);

    let (
        previous_state,
        previous_state_hash,
        previous_blockchain_proof_input,
        previous_state_body_hash,
    ) = {
        w.exists(prev_state);

        let (previous_state_hash, body) = checked_hash_protocol_state(prev_state, w);

        (prev_state, previous_state_hash, (), body)
    };

    // let statement: Statement<()> = statement.into();
    // let sok_digest = message.digest();
    // let statement_with_sok = statement.with_digest(sok_digest);

    // w.exists(&statement_with_sok);
}
