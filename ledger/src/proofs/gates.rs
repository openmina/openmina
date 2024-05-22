use std::{collections::HashMap, path::Path, sync::Arc};

use kimchi::circuits::gate::CircuitGate;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use once_cell::sync::Lazy;

use super::{
    constants::{
        StepBlockProof, StepMergeProof, StepTransactionProof, StepZkappOptSignedOptSignedProof,
        StepZkappOptSignedProof, StepZkappProvedProof, WrapBlockProof, WrapTransactionProof,
    },
    field::FieldWitness,
    transaction::{make_prover_index, InternalVars, Prover, V},
};

use mina_p2p_messages::binprot::{
    self,
    macros::{BinProtRead, BinProtWrite},
};

struct Gates {
    step_tx_gates: Vec<CircuitGate<Fp>>,
    wrap_tx_gates: Vec<CircuitGate<Fq>>,
    step_merge_gates: Vec<CircuitGate<Fp>>,
    step_block_gates: Vec<CircuitGate<Fp>>,
    wrap_block_gates: Vec<CircuitGate<Fq>>,
    step_opt_signed_opt_signed_gates: Vec<CircuitGate<Fp>>,
    step_opt_signed_gates: Vec<CircuitGate<Fp>>,
    step_proved_gates: Vec<CircuitGate<Fp>>,
    step_tx_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    step_tx_rows_rev: Vec<Vec<Option<V>>>,
    wrap_tx_internal_vars: HashMap<usize, (Vec<(Fq, V)>, Option<Fq>)>,
    wrap_tx_rows_rev: Vec<Vec<Option<V>>>,
    step_merge_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    step_merge_rows_rev: Vec<Vec<Option<V>>>,
    step_block_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    step_block_rows_rev: Vec<Vec<Option<V>>>,
    wrap_block_internal_vars: HashMap<usize, (Vec<(Fq, V)>, Option<Fq>)>,
    wrap_block_rows_rev: Vec<Vec<Option<V>>>,
    step_opt_signed_opt_signed_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    step_opt_signed_opt_signed_rows_rev: Vec<Vec<Option<V>>>,
    step_opt_signed_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    step_opt_signed_rows_rev: Vec<Vec<Option<V>>>,
    step_proved_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    step_proved_rows_rev: Vec<Vec<Option<V>>>,
}

fn read_gates() -> Gates {
    fn read_gates_file<F: FieldWitness>(
        filepath: &impl AsRef<Path>,
    ) -> std::io::Result<Vec<CircuitGate<F>>> {
        use serde_with::serde_as;

        #[serde_as]
        #[derive(serde::Deserialize)]
        struct GatesFile<F: ark_ff::PrimeField> {
            public_input_size: usize,
            #[serde_as(as = "Vec<_>")]
            gates: Vec<CircuitGate<F>>,
        }

        let file = std::fs::File::open(filepath)?;
        let reader = std::io::BufReader::new(file);
        let data: GatesFile<F> = serde_json::from_reader(reader)?;
        Ok(data.gates)
    }

    fn make<F: FieldWitness>(
        filename: &str,
    ) -> (
        HashMap<usize, (Vec<(F, V)>, Option<F>)>,
        Vec<Vec<Option<V>>>,
        Vec<CircuitGate<F>>,
    ) {
        let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let base_dir = base_dir.join("3.0.0devnet");

        let internal_vars_path = base_dir.join(format!("{}_internal_vars.bin", filename));
        let rows_rev_path = base_dir.join(format!("{}_rows_rev.bin", filename));
        let gates_path = base_dir.join(format!("{}_gates.json", filename));

        let gates: Vec<CircuitGate<F>> = read_gates_file(&gates_path).unwrap();
        let (internal_vars_path, rows_rev_path) =
            read_constraints_data::<F>(&internal_vars_path, &rows_rev_path).unwrap();

        (internal_vars_path, rows_rev_path, gates)
    }

    let (step_tx_internal_vars, step_tx_rows_rev, step_tx_gates) = {
        make("step-step-proving-key-transaction-snark-transaction-0-81cc493d6bc2538fdbee3ad60fd77758")
    };
    let (wrap_tx_internal_vars, wrap_tx_rows_rev, wrap_tx_gates) =
        { make("wrap-wrap-proving-key-transaction-snark-96f388cb62fd3b955368b475623e0a92") };
    let (step_merge_internal_vars, step_merge_rows_rev, step_merge_gates) = {
        make("step-step-proving-key-transaction-snark-merge-1-ba1d52dfdc2dd4d2e61f6c66ff2a5b2f")
    };
    let (step_block_internal_vars, step_block_rows_rev, step_block_gates) =
        { make("step-step-proving-key-blockchain-snark-step-0-c6715547791ab80660cda1f715ce8c58") };
    let (wrap_block_internal_vars, wrap_block_rows_rev, wrap_block_gates) =
        { make("wrap-wrap-proving-key-blockchain-snark-b18a44f63a978aec2c3f3dbb392acbfb") };
    let (
        step_opt_signed_opt_signed_internal_vars,
        step_opt_signed_opt_signed_rows_rev,
        step_opt_signed_opt_signed_gates,
    ) = {
        make("step-step-proving-key-transaction-snark-opt_signed-opt_signed-2-4a4a49c139f2fb195603ffc386c41cc6")
    };
    let (step_opt_signed_internal_vars, step_opt_signed_rows_rev, step_opt_signed_gates) = {
        make(
            "step-step-proving-key-transaction-snark-opt_signed-3-6f4b0c362fb64d33fe3c8a3ed1351de5",
        )
    };
    let (step_proved_internal_vars, step_proved_rows_rev, step_proved_gates) = {
        make("step-step-proving-key-transaction-snark-proved-4-879547e44319e1b0a4bb2513c66be9f8")
    };

    Gates {
        step_tx_gates,
        wrap_tx_gates,
        step_merge_gates,
        step_block_gates,
        step_tx_internal_vars,
        step_tx_rows_rev,
        wrap_tx_internal_vars,
        wrap_tx_rows_rev,
        step_merge_internal_vars,
        step_merge_rows_rev,
        step_block_internal_vars,
        step_block_rows_rev,
        wrap_block_gates,
        wrap_block_internal_vars,
        wrap_block_rows_rev,
        step_opt_signed_opt_signed_gates,
        step_opt_signed_opt_signed_internal_vars,
        step_opt_signed_opt_signed_rows_rev,
        step_opt_signed_gates,
        step_opt_signed_internal_vars,
        step_opt_signed_rows_rev,
        step_proved_gates,
        step_proved_internal_vars,
        step_proved_rows_rev,
    }
}

static PROVERS: Lazy<Arc<Provers>> = Lazy::new(|| Arc::new(make_provers()));

pub struct Provers {
    pub tx_step_prover: Prover<Fp>,
    pub tx_wrap_prover: Prover<Fq>,
    pub merge_step_prover: Prover<Fp>,
    pub block_step_prover: Prover<Fp>,
    pub block_wrap_prover: Prover<Fq>,
    pub zkapp_step_opt_signed_opt_signed_prover: Prover<Fp>,
    pub zkapp_step_opt_signed_prover: Prover<Fp>,
    pub zkapp_step_proof_prover: Prover<Fp>,
}

pub fn get_provers() -> Arc<Provers> {
    Arc::clone(&*PROVERS)
}

/// Slow, use `get_provers` instead
fn make_provers() -> Provers {
    let Gates {
        step_tx_gates,
        wrap_tx_gates,
        step_merge_gates,
        step_block_gates,
        step_tx_internal_vars,
        step_tx_rows_rev,
        wrap_tx_internal_vars,
        wrap_tx_rows_rev,
        step_merge_internal_vars,
        step_merge_rows_rev,
        step_block_internal_vars,
        step_block_rows_rev,
        wrap_block_gates,
        wrap_block_internal_vars,
        wrap_block_rows_rev,
        step_opt_signed_opt_signed_gates,
        step_opt_signed_opt_signed_internal_vars,
        step_opt_signed_opt_signed_rows_rev,
        step_opt_signed_gates,
        step_opt_signed_internal_vars,
        step_opt_signed_rows_rev,
        step_proved_gates,
        step_proved_internal_vars,
        step_proved_rows_rev,
    } = read_gates();

    let tx_prover_index = make_prover_index::<StepTransactionProof, _>(step_tx_gates);
    let merge_prover_index = make_prover_index::<StepMergeProof, _>(step_merge_gates);
    let wrap_prover_index = make_prover_index::<WrapTransactionProof, _>(wrap_tx_gates);
    let wrap_block_prover_index = make_prover_index::<WrapBlockProof, _>(wrap_block_gates);
    let block_prover_index = make_prover_index::<StepBlockProof, _>(step_block_gates);
    let zkapp_step_opt_signed_opt_signed_prover_index =
        make_prover_index::<StepZkappOptSignedOptSignedProof, _>(step_opt_signed_opt_signed_gates);
    let zkapp_step_opt_signed_prover_index =
        make_prover_index::<StepZkappOptSignedProof, _>(step_opt_signed_gates);
    let zkapp_step_proof_prover_index =
        make_prover_index::<StepZkappProvedProof, _>(step_proved_gates);

    let tx_step_prover = Prover {
        internal_vars: step_tx_internal_vars,
        rows_rev: step_tx_rows_rev,
        index: tx_prover_index,
    };

    let merge_step_prover = Prover {
        internal_vars: step_merge_internal_vars,
        rows_rev: step_merge_rows_rev,
        index: merge_prover_index,
    };

    let tx_wrap_prover = Prover {
        internal_vars: wrap_tx_internal_vars,
        rows_rev: wrap_tx_rows_rev,
        index: wrap_prover_index,
    };

    let block_step_prover = Prover {
        internal_vars: step_block_internal_vars,
        rows_rev: step_block_rows_rev,
        index: block_prover_index,
    };

    let block_wrap_prover = Prover {
        internal_vars: wrap_block_internal_vars,
        rows_rev: wrap_block_rows_rev,
        index: wrap_block_prover_index,
    };

    let zkapp_step_opt_signed_opt_signed_prover = Prover {
        internal_vars: step_opt_signed_opt_signed_internal_vars,
        rows_rev: step_opt_signed_opt_signed_rows_rev,
        index: zkapp_step_opt_signed_opt_signed_prover_index,
    };

    let zkapp_step_opt_signed_prover = Prover {
        internal_vars: step_opt_signed_internal_vars,
        rows_rev: step_opt_signed_rows_rev,
        index: zkapp_step_opt_signed_prover_index,
    };

    let zkapp_step_proof_prover = Prover {
        internal_vars: step_proved_internal_vars,
        rows_rev: step_proved_rows_rev,
        index: zkapp_step_proof_prover_index,
    };

    Provers {
        tx_step_prover,
        tx_wrap_prover,
        merge_step_prover,
        block_step_prover,
        block_wrap_prover,
        zkapp_step_opt_signed_opt_signed_prover,
        zkapp_step_opt_signed_prover,
        zkapp_step_proof_prover,
    }
}

pub fn read_constraints_data<F: FieldWitness>(
    internal_vars_path: &Path,
    rows_rev_path: &Path,
) -> Option<(InternalVars<F>, Vec<Vec<Option<V>>>)> {
    use mina_p2p_messages::bigint::BigInt;

    impl From<&VRaw> for V {
        fn from(value: &VRaw) -> Self {
            match value {
                VRaw::External(index) => Self::External(*index as usize),
                VRaw::Internal(id) => Self::Internal(*id as usize),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
    enum VRaw {
        External(u32),
        Internal(u32),
    }

    use binprot::BinProtRead;

    type InternalVarsRaw = HashMap<u32, (Vec<(BigInt, VRaw)>, Option<BigInt>)>;

    // ((Fp.t * V.t) list * Fp.t option)
    let Ok(internal_vars) = std::fs::read(internal_vars_path)
    // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("internal_vars.bin"))
    else {
        return None;
    };
    let internal_vars: InternalVarsRaw =
        HashMap::binprot_read(&mut internal_vars.as_slice()).unwrap();

    // V.t option array list
    let rows_rev = std::fs::read(rows_rev_path).unwrap();
    // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rows_rev.bin")).unwrap();
    let rows_rev: Vec<Vec<Option<VRaw>>> = Vec::binprot_read(&mut rows_rev.as_slice()).unwrap();

    let internal_vars: InternalVars<F> = internal_vars
        .into_iter()
        .map(|(id, (list, opt))| {
            let id = id as usize;
            let list: Vec<_> = list
                .iter()
                .map(|(n, v)| (n.to_field::<F>(), V::from(v)))
                .collect();
            let opt = opt.as_ref().map(BigInt::to_field::<F>);
            (id, (list, opt))
        })
        .collect();

    let rows_rev: Vec<_> = rows_rev
        .iter()
        .map(|row| {
            let row: Vec<_> = row.iter().map(|v| v.as_ref().map(V::from)).collect();
            row
        })
        .collect();

    Some((internal_vars, rows_rev))
}
