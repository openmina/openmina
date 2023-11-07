use std::{path::Path, str::FromStr, sync::Arc};

use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    dummy,
    proofs::{numbers::currency::CheckedSigned, witness::Boolean},
    scan_state::{
        currency::Sgn,
        protocol_state::MinaHash,
        scan_state::transaction_snark::{Registers, SokDigest, Statement},
    },
    Inputs, ToInputs,
};

use super::{
    numbers::currency::{CheckedAmount, CheckedFee},
    to_field_elements::ToFieldElements,
    witness::{checked_hash2, field, transaction_snark::checked_hash, Check, Prover, Witness},
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

fn non_pc_registers_equal_var(t1: &Registers, t2: &Registers, w: &mut Witness<Fp>) -> Boolean {
    let alls = [
        // t1.pending_coinbase_stack.equal_var(&t2.pending_coinbase_stack, w),
        field::equal(t1.first_pass_ledger, t2.first_pass_ledger, w),
        field::equal(t1.second_pass_ledger, t2.second_pass_ledger, w),
    ]
    .into_iter()
    .chain(t1.local_state.checked_equal_prime(&t2.local_state, w))
    .collect::<Vec<_>>();

    Boolean::all(&alls, w)
}

fn txn_statement_ledger_hashes_equal(
    s1: &Statement<()>,
    s2: &Statement<()>,
    w: &mut Witness<Fp>,
) -> Boolean {
    let source_eq = non_pc_registers_equal_var(&s1.source, &s2.source, w);
    let target_eq = non_pc_registers_equal_var(&s1.target, &s2.target, w);
    let left_ledger_eq = field::equal(s1.connecting_ledger_left, s2.connecting_ledger_left, w);
    let right_ledger_eq = field::equal(s1.connecting_ledger_right, s2.connecting_ledger_right, w);
    let supply_increase_eq = s1
        .supply_increase
        .to_checked()
        .equal(&s2.supply_increase.to_checked(), w);

    Boolean::all(
        &[
            source_eq,
            target_eq,
            left_ledger_eq,
            right_ledger_eq,
            supply_increase_eq,
        ],
        w,
    )
}

fn consensus_state_next_state_checked(
    prev_state: &v2::MinaStateProtocolStateValueStableV2,
    prev_state_hash: Fp,
    transition: &v2::MinaStateSnarkTransitionValueStableV2,
    supply_increase: CheckedSigned<Fp, CheckedAmount<Fp>>,
    w: &mut Witness<Fp>,
) {
    let protocol_constants = &prev_state.body.constants;
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

    let txn_stmt_ledger_hashes_didn_t_change = {
        let s1: Statement<()> =
            (&previous_state.body.blockchain_state.ledger_proof_statement).into();
        let s2: Statement<()> = txn_snark.clone().without_digest();
        txn_statement_ledger_hashes_equal(&s1, &s2, w)
    };

    eprintln!("AAA");

    let supply_increase = w.exists_no_check(match txn_stmt_ledger_hashes_didn_t_change {
        Boolean::True => CheckedSigned::zero(),
        Boolean::False => txn_snark.supply_increase.to_checked(),
    });

    consensus_state_next_state_checked(
        previous_state,
        previous_state_hash,
        transition,
        supply_increase,
    );

    // let%bind supply_increase =
    //   (* only increase the supply if the txn statement represents a new ledger transition *)
    //   Currency.Amount.(
    //     Signed.Checked.if_ txn_stmt_ledger_hashes_didn't_change
    //       ~then_:
    //         (Signed.create_var ~magnitude:(var_of_t zero) ~sgn:Sgn.Checked.pos)
    //       ~else_:txn_snark.supply_increase)
    // in
    // let%bind `Success updated_consensus_state, consensus_state =
    //   with_label __LOC__ (fun () ->
    //       Consensus_state_hooks.next_state_checked ~constraint_constants
    //         ~prev_state:previous_state ~prev_state_hash:previous_state_hash
    //         transition supply_increase )
    // in
    // let global_slot =
    //   Consensus.Data.Consensus_state.global_slot_since_genesis_var consensus_state
    // in
    // let supercharge_coinbase =
    //   Consensus.Data.Consensus_state.supercharge_coinbase_var consensus_state
    // in
    // let prev_pending_coinbase_root =
    //   previous_state |> Protocol_state.blockchain_state
    //   |> Blockchain_state.staged_ledger_hash
    //   |> Staged_ledger_hash.pending_coinbase_hash_var
    // in
}
