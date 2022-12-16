use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, LedgerProofProdStableV2,
    TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    TransactionSnarkStatementWithSokStableV2,
};
use mina_signer::CompressedPubKey;

use crate::scan_state::{
    currency::{Amount, Signed},
    fee_excess::FeeExcess,
};

use super::parallel_scan::ParallelScan;
// use super::parallel_scan::AvailableJob;

pub use super::parallel_scan::SpacePartition;

type LedgerProof = LedgerProofProdStableV2;
type LedgerProofWithSokMessage = TransactionSnarkScanStateLedgerProofWithSokMessageStableV2;
// type TransactionWithWitness = TransactionSnarkScanStateTransactionWithWitnessStableV2;

pub type AvailableJob = super::parallel_scan::AvailableJob<
    transaction_snark::TransactionWithWitness,
    LedgerProofWithSokMessage,
>;

pub struct ScanState {
    state: ParallelScan<transaction_snark::TransactionWithWitness, LedgerProofWithSokMessage>,
}

pub mod transaction_snark {
    use mina_hasher::Fp;
    use mina_p2p_messages::v2::{
        MinaBasePendingCoinbaseStackVersionedStableV1, MinaBaseSparseLedgerBaseStableV2,
        MinaBaseStateBodyHashStableV1, MinaTransactionLogicTransactionAppliedStableV2,
        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1, StateHash,
        TransactionSnarkPendingCoinbaseStackStateInitStackStableV1,
    };

    use crate::scan_state::{
        currency::{Amount, Signed},
        fee_excess::FeeExcess,
    };

    type LedgerHash = Fp;

    #[derive(Debug, Clone)]
    pub struct Source {
        pub ledger: LedgerHash,
        pub pending_coinbase_stack: MinaBasePendingCoinbaseStackVersionedStableV1,
        pub local_state: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
    }

    #[derive(Debug, Clone)]
    pub struct Statement {
        pub source: Source,
        pub target: Source,
        pub supply_increase: Signed<Amount>,
        pub fee_excess: FeeExcess,
        pub sok_digest: (),
    }

    #[derive(Debug, Clone)]
    pub struct TransactionWithWitness {
        pub transaction_with_info: MinaTransactionLogicTransactionAppliedStableV2,
        pub state_hash: (StateHash, MinaBaseStateBodyHashStableV1),
        pub statement: Statement,
        pub init_stack: TransactionSnarkPendingCoinbaseStackStateInitStackStableV1,
        pub ledger_witness: MinaBaseSparseLedgerBaseStableV2,
    }
}

impl ScanState {
    pub fn hash(&self) {
        use binprot::BinProtWrite;

        self.state.hash(
            |buffer, proof| {
                proof.binprot_write(buffer).unwrap();
            },
            |buffer, transaction| {
                transaction.binprot_write(buffer).unwrap();
            },
        );
    }
}

pub struct ForkConstants {
    previous_state_hash: Fp,   // Pickles.Backend.Tick.Field.Stable.Latest.t,
    previous_length: u32,      // Mina_numbers.Length.Stable.Latest.t,
    previous_global_slot: u32, // Mina_numbers.Global_slot.Stable.Latest.t,
}

pub struct GenesisConstants {
    sub_windows_per_window: u64,
    ledger_depth: u64,
    work_delay: u64,
    block_window_duration_ms: u64,
    transaction_capacity_log_2: u64,
    pending_coinbase_depth: u64,
    coinbase_amount: u64, // Currency.Amount.Stable.Latest.t,
    supercharged_coinbase_factor: u64,
    account_creation_fee: u64,   // Currency.Fee.Stable.Latest.t,
    fork: Option<ForkConstants>, // Fork_constants.t option,
}

struct Fee(u64);

fn completed_work_to_scanable_work(
    job: AvailableJob,
    (fee, current_proof, prover): (CurrencyFeeStableV1, LedgerProof, CompressedPubKey),
) -> LedgerProofWithSokMessage {
    use super::parallel_scan::AvailableJob::{Base, Merge};
    use transaction_snark::TransactionWithWitness;

    let sok_digest = &current_proof.0.statement.sok_digest.0;
    let proof = &current_proof.0.proof;

    match job {
        Base(TransactionWithWitness { statement, .. }) => {
            todo!()
            // let statement_with_sok = TransactionSnarkStatementWithSokStableV2 {
            //     source: statement.source,
            //     target: statement.target,
            //     supply_increase: statement.supply_increase,
            //     fee_excess: statement.fee_excess,
            //     sok_digest: MinaBaseSokMessageDigestStableV1(sok_digest.clone()),
            // };

            // let ledger_proof = LedgerProofProdStableV2(TransactionSnarkStableV2 {
            //     statement: statement_with_sok,
            //     proof: proof.clone(),
            // });

            // let prover: NonZeroCurvePointUncompressedStableV1 = prover.into();
            // let sok = MinaBaseSokMessageStableV1 {
            //     fee,
            //     prover: prover.into(),
            // };

            // TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(ledger_proof, sok)
        }
        Merge {
            left: proof1,
            right: proof2,
        } => {
            let s1: &TransactionSnarkStatementWithSokStableV2 = &proof1.0 .0.statement;
            let s2: &TransactionSnarkStatementWithSokStableV2 = &proof2.0 .0.statement;

            let s1_fee_excess: FeeExcess = (&s1.fee_excess).into();
            let s2_fee_excess: FeeExcess = (&s2.fee_excess).into();

            let fee_excess = FeeExcess::combine(&s1_fee_excess, &s2_fee_excess);

            let s1_supply_increase: Signed<Amount> = (&s1.supply_increase).into();
            let s2_supply_increase: Signed<Amount> = (&s2.supply_increase).into();

            let supply_increase = s1_supply_increase
                .add(&s2_supply_increase)
                .expect("Error adding supply_increases");

            if s1.target.pending_coinbase_stack != s2.source.pending_coinbase_stack {
                panic!("Invalid pending coinbase stack state");
            }

            // let statement = TransactionSnarkStatementStableV2 {
            //     source: s1.source.clone(),
            //     target: s2.target.clone(),
            //     supply_increase,
            //     fee_excess,
            //     sok_digest: (),
            // };

            todo!()
        }
    }
}

// let completed_work_to_scanable_work (job : job) (fee, current_proof, prover) :
//     Ledger_proof_with_sok_message.t Or_error.t =
//   let sok_digest = Ledger_proof.sok_digest current_proof
//   and proof = Ledger_proof.underlying_proof current_proof in
//   match job with
//   | Base { statement; _ } ->
//       let ledger_proof = Ledger_proof.create ~statement ~sok_digest ~proof in
//       Ok (ledger_proof, Sok_message.create ~fee ~prover)
//   | Merge ((p, _), (p', _)) ->
//       let open Or_error.Let_syntax in
//       (*
//       let%map statement =
//         Transaction_snark.Statement.merge (Ledger_proof.statement p)
//           (Ledger_proof.statement p')
//       in *)
//       let s = Ledger_proof.statement p and s' = Ledger_proof.statement p' in
//       let option lab =
//         Option.value_map ~default:(Or_error.error_string lab) ~f:(fun x -> Ok x)
//       in
//       let%map fee_excess = Fee_excess.combine s.fee_excess s'.fee_excess
//       and supply_increase =
//         Amount.Signed.add s.supply_increase s'.supply_increase
//         |> option "Error adding supply_increases"
//       and _valid_pending_coinbase_stack =
//         if
//           Pending_coinbase.Stack.equal s.target.pending_coinbase_stack
//             s'.source.pending_coinbase_stack
//         then Ok ()
//         else Or_error.error_string "Invalid pending coinbase stack state"
//       in
//       let statement : Transaction_snark.Statement.t =
//         { source = s.source
//         ; target = s'.target
//         ; supply_increase
//         ; fee_excess
//         ; sok_digest = ()
//         }
//       in
//       ( Ledger_proof.create ~statement ~sok_digest ~proof
//       , Sok_message.create ~fee ~prover )
