use std::{collections::HashMap, path::Path, sync::Arc};

use kimchi::circuits::gate::CircuitGate;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use once_cell::sync::Lazy;

use super::{
    constants::{
        StepBlockProof, StepMergeProof, StepTransactionProof, StepZkappOptSignedOptSignedProof,
        StepZkappOptSignedProof, StepZkappProofProof, WrapBlockProof, WrapTransactionProof,
    },
    field::FieldWitness,
    transaction::{make_prover_index, InternalVars, Prover, V},
};

use mina_p2p_messages::binprot::{
    self,
    macros::{BinProtRead, BinProtWrite},
};

struct Gates {
    gates: Vec<CircuitGate<Fp>>,
    wrap_gates: Vec<CircuitGate<Fq>>,
    merge_gates: Vec<CircuitGate<Fp>>,
    block_gates: Vec<CircuitGate<Fp>>,
    block_wrap_gates: Vec<CircuitGate<Fq>>,
    zkapp_step_opt_signed_opt_signed_gates: Vec<CircuitGate<Fp>>,
    zkapp_step_opt_signed_gates: Vec<CircuitGate<Fp>>,
    zkapp_step_proof_gates: Vec<CircuitGate<Fp>>,
    internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    rows_rev: Vec<Vec<Option<V>>>,
    internal_vars_wrap: HashMap<usize, (Vec<(Fq, V)>, Option<Fq>)>,
    rows_rev_wrap: Vec<Vec<Option<V>>>,
    merge_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    merge_rows_rev: Vec<Vec<Option<V>>>,
    block_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    block_rows_rev: Vec<Vec<Option<V>>>,
    block_wrap_internal_vars: HashMap<usize, (Vec<(Fq, V)>, Option<Fq>)>,
    block_wrap_rows_rev: Vec<Vec<Option<V>>>,
    zkapp_step_opt_signed_opt_signed_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    zkapp_step_opt_signed_opt_signed_rows_rev: Vec<Vec<Option<V>>>,
    zkapp_step_opt_signed_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    zkapp_step_opt_signed_rows_rev: Vec<Vec<Option<V>>>,
    zkapp_step_proof_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
    zkapp_step_proof_rows_rev: Vec<Vec<Option<V>>>,
}

fn read_gates() -> Gates {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let base_dir = base_dir.join("rampup4");

    fn read_gates_file<F: FieldWitness>(
        filepath: &impl AsRef<Path>,
    ) -> std::io::Result<Vec<CircuitGate<F>>> {
        let file = std::fs::File::open(filepath)?;
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).map_err(Into::into)
    }

    let internal_vars_path = base_dir.join("internal_vars_rampup4.bin");
    let rows_rev_path = base_dir.join("rows_rev_rampup4.bin");
    let (internal_vars, rows_rev) =
        read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();

    let internal_vars_path = base_dir.join("internal_vars_wrap_rampup4.bin");
    let rows_rev_path = base_dir.join("rows_rev_wrap_rampup4.bin");
    let (internal_vars_wrap, rows_rev_wrap) =
        read_constraints_data::<Fq>(&internal_vars_path, &rows_rev_path).unwrap();

    let internal_vars_path = base_dir.join("merge_internal_vars.bin");
    let rows_rev_path = base_dir.join("merge_rows_rev.bin");
    let (merge_internal_vars, merge_rows_rev) =
        read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();

    let internal_vars_path = base_dir.join("block_internal_vars.bin");
    let rows_rev_path = base_dir.join("block_rows_rev.bin");
    let (block_internal_vars, block_rows_rev) =
        read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();

    let internal_vars_path = base_dir.join("block_wrap_internal_vars.bin");
    let rows_rev_path = base_dir.join("block_wrap_rows_rev.bin");
    let (block_wrap_internal_vars, block_wrap_rows_rev) =
        read_constraints_data::<Fq>(&internal_vars_path, &rows_rev_path).unwrap();

    let internal_vars_path = base_dir.join("zkapp_step_internal_vars.bin");
    let rows_rev_path = base_dir.join("zkapp_step_rows_rev.bin");
    let (zkapp_step_opt_signed_opt_signed_internal_vars, zkapp_step_opt_signed_opt_signed_rows_rev) =
        read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();

    let internal_vars_path = base_dir.join("zkapp_step_opt_signed_internal_vars.bin");
    let rows_rev_path = base_dir.join("zkapp_step_opt_signed_rows_rev.bin");
    let (zkapp_step_opt_signed_internal_vars, zkapp_step_opt_signed_rows_rev) =
        read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();

    let gates: Vec<CircuitGate<Fp>> =
        read_gates_file(&base_dir.join("gates_step_rampup4.json")).unwrap();
    let wrap_gates: Vec<CircuitGate<Fq>> =
        read_gates_file(&base_dir.join("gates_wrap_rampup4.json")).unwrap();
    let merge_gates: Vec<CircuitGate<Fp>> =
        read_gates_file(&base_dir.join("gates_merge_rampup4.json")).unwrap();

    let block_gates: Vec<CircuitGate<Fp>> =
        read_gates_file(&base_dir.join("block_gates.json")).unwrap();
    let block_wrap_gates: Vec<CircuitGate<Fq>> =
        read_gates_file(&base_dir.join("block_wrap_gates.json")).unwrap();
    let zkapp_step_opt_signed_opt_signed_gates: Vec<CircuitGate<Fp>> =
        read_gates_file(&base_dir.join("zkapp_step_gates.json")).unwrap();
    let zkapp_step_opt_signed_gates: Vec<CircuitGate<Fp>> =
        read_gates_file(&base_dir.join("zkapp_step_opt_signed_gates.json")).unwrap();

    let internal_vars_path = base_dir.join("zkapp_step_proof_internal_vars.bin");
    let rows_rev_path = base_dir.join("zkapp_step_proof_rows_rev.bin");
    let (zkapp_step_proof_internal_vars, zkapp_step_proof_rows_rev) =
        read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();
    let zkapp_step_proof_gates: Vec<CircuitGate<Fp>> =
        read_gates_file(&base_dir.join("zkapp_step_proof_gates.json")).unwrap();

    Gates {
        gates,
        wrap_gates,
        merge_gates,
        block_gates,
        internal_vars,
        rows_rev,
        internal_vars_wrap,
        rows_rev_wrap,
        merge_internal_vars,
        merge_rows_rev,
        block_internal_vars,
        block_rows_rev,
        block_wrap_gates,
        block_wrap_internal_vars,
        block_wrap_rows_rev,
        zkapp_step_opt_signed_opt_signed_gates,
        zkapp_step_opt_signed_opt_signed_internal_vars,
        zkapp_step_opt_signed_opt_signed_rows_rev,
        zkapp_step_opt_signed_gates,
        zkapp_step_opt_signed_internal_vars,
        zkapp_step_opt_signed_rows_rev,
        zkapp_step_proof_gates,
        zkapp_step_proof_internal_vars,
        zkapp_step_proof_rows_rev,
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
        gates,
        wrap_gates,
        merge_gates,
        block_gates,
        internal_vars,
        rows_rev,
        internal_vars_wrap,
        rows_rev_wrap,
        merge_internal_vars,
        merge_rows_rev,
        block_internal_vars,
        block_rows_rev,
        block_wrap_gates,
        block_wrap_internal_vars,
        block_wrap_rows_rev,
        zkapp_step_opt_signed_opt_signed_gates,
        zkapp_step_opt_signed_opt_signed_internal_vars,
        zkapp_step_opt_signed_opt_signed_rows_rev,
        zkapp_step_opt_signed_gates,
        zkapp_step_opt_signed_internal_vars,
        zkapp_step_opt_signed_rows_rev,
        zkapp_step_proof_gates,
        zkapp_step_proof_internal_vars,
        zkapp_step_proof_rows_rev,
    } = read_gates();

    let tx_prover_index = make_prover_index::<StepTransactionProof, _>(gates);
    let merge_prover_index = make_prover_index::<StepMergeProof, _>(merge_gates);
    let wrap_prover_index = make_prover_index::<WrapTransactionProof, _>(wrap_gates);
    let wrap_block_prover_index = make_prover_index::<WrapBlockProof, _>(block_wrap_gates);
    let block_prover_index = make_prover_index::<StepBlockProof, _>(block_gates);
    let zkapp_step_opt_signed_opt_signed_prover_index =
        make_prover_index::<StepZkappOptSignedOptSignedProof, _>(
            zkapp_step_opt_signed_opt_signed_gates,
        );
    let zkapp_step_opt_signed_prover_index =
        make_prover_index::<StepZkappOptSignedProof, _>(zkapp_step_opt_signed_gates);
    let zkapp_step_proof_prover_index =
        make_prover_index::<StepZkappProofProof, _>(zkapp_step_proof_gates);

    let tx_step_prover = Prover {
        internal_vars,
        rows_rev,
        index: tx_prover_index,
    };

    let merge_step_prover = Prover {
        internal_vars: merge_internal_vars,
        rows_rev: merge_rows_rev,
        index: merge_prover_index,
    };

    let tx_wrap_prover = Prover {
        internal_vars: internal_vars_wrap,
        rows_rev: rows_rev_wrap,
        index: wrap_prover_index,
    };

    let block_step_prover = Prover {
        internal_vars: block_internal_vars,
        rows_rev: block_rows_rev,
        index: block_prover_index,
    };

    let block_wrap_prover = Prover {
        internal_vars: block_wrap_internal_vars,
        rows_rev: block_wrap_rows_rev,
        index: wrap_block_prover_index,
    };

    let zkapp_step_opt_signed_opt_signed_prover = Prover {
        internal_vars: zkapp_step_opt_signed_opt_signed_internal_vars,
        rows_rev: zkapp_step_opt_signed_opt_signed_rows_rev,
        index: zkapp_step_opt_signed_opt_signed_prover_index,
    };

    let zkapp_step_opt_signed_prover = Prover {
        internal_vars: zkapp_step_opt_signed_internal_vars,
        rows_rev: zkapp_step_opt_signed_rows_rev,
        index: zkapp_step_opt_signed_prover_index,
    };

    let zkapp_step_proof_prover = Prover {
        internal_vars: zkapp_step_proof_internal_vars,
        rows_rev: zkapp_step_proof_rows_rev,
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
