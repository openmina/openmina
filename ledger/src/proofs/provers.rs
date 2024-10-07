use std::{collections::HashMap, path::Path, sync::Arc};

use kimchi::circuits::gate::CircuitGate;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use once_cell::sync::OnceCell;
use openmina_core::network::CircuitsConfig;

use super::{
    circuit_blobs,
    constants::{
        ProofConstants, StepBlockProof, StepMergeProof, StepTransactionProof,
        StepZkappOptSignedOptSignedProof, StepZkappOptSignedProof, StepZkappProvedProof,
        WrapBlockProof, WrapTransactionProof,
    },
    field::FieldWitness,
    transaction::{make_prover_index, InternalVars, Prover, V},
    verifiers::{BlockVerifier, TransactionVerifier},
};

use mina_p2p_messages::binprot::{
    self,
    macros::{BinProtRead, BinProtWrite},
};

pub const fn devnet_circuit_directory() -> &'static str {
    openmina_core::network::devnet::CIRCUITS_CONFIG.directory_name
}

// TODO(tizoc): right now all tests are for devnets, and the above
// function is used for those tests.
// pub fn mainnet_circuit_directory() -> &'static str {
//     openmina_core::network::devnet::CIRCUITS_CONFIG.directory_name
// }
//
// pub fn circuit_directory() -> &'static str {
//     openmina_core::NetworkConfig::global().circuits_config.directory_name
// }

fn decode_gates_file<F: FieldWitness>(
    reader: impl std::io::Read,
) -> std::io::Result<Vec<CircuitGate<F>>> {
    #[serde_with::serde_as]
    #[derive(serde::Deserialize)]
    struct GatesFile<F: ark_ff::PrimeField> {
        public_input_size: usize,
        #[serde_as(as = "Vec<_>")]
        gates: Vec<CircuitGate<F>>,
    }
    let data: GatesFile<F> = serde_json::from_reader(reader)?;
    Ok(data.gates)
}

async fn read_gates_file<F: FieldWitness>(
    filepath: &impl AsRef<Path>,
) -> std::io::Result<Vec<CircuitGate<F>>> {
    let resp = circuit_blobs::fetch(filepath).await?;
    decode_gates_file(&mut resp.as_slice())
}

async fn make_gates<F: FieldWitness>(
    filename: &str,
) -> (
    HashMap<usize, (Vec<(F, V)>, Option<F>)>,
    Vec<Vec<Option<V>>>,
    Vec<CircuitGate<F>>,
) {
    let circuits_config = openmina_core::NetworkConfig::global().circuits_config;
    let base_dir = Path::new(circuits_config.directory_name);

    let internal_vars_path = base_dir.join(format!("{}_internal_vars.bin", filename));
    let rows_rev_path = base_dir.join(format!("{}_rows_rev.bin", filename));
    let gates_path = base_dir.join(format!("{}_gates.json", filename));

    let gates: Vec<CircuitGate<F>> = read_gates_file(&gates_path).await.unwrap();
    let (internal_vars_path, rows_rev_path) =
        read_constraints_data::<F>(&internal_vars_path, &rows_rev_path)
            .await
            .unwrap();

    (internal_vars_path, rows_rev_path, gates)
}

async fn get_or_make<T: ProofConstants, F: FieldWitness>(
    constant: &OnceCell<Arc<Prover<F>>>,
    verifier_index: Option<Arc<super::VerifierIndex<F>>>,
    filename: &str,
) -> Arc<Prover<F>> {
    if let Some(prover) = constant.get() {
        return prover.clone();
    }

    let (internal_vars, rows_rev, gates) = { make_gates(filename).await };

    let index = make_prover_index::<T, _>(gates, verifier_index);
    let prover = Prover {
        internal_vars,
        rows_rev,
        index,
    };

    constant.get_or_init(|| prover.into()).clone()
}

/// Run the future, this does not support awakes
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    use std::task::{Context, Poll, Wake, Waker};

    struct NoAwake;
    impl Wake for NoAwake {
        fn wake(self: Arc<Self>) {
            unreachable!() // We don't support that
        }
    }

    let mut future = std::pin::pin!(future);
    let waker = Waker::from(Arc::new(NoAwake));
    match future.as_mut().poll(&mut Context::from_waker(&waker)) {
        Poll::Ready(result) => return result,
        Poll::Pending => unreachable!(), // We don't support that
    }
}

static TX_STEP_PROVER: OnceCell<Arc<Prover<Fp>>> = OnceCell::new();
static TX_WRAP_PROVER: OnceCell<Arc<Prover<Fq>>> = OnceCell::new();
static MERGE_STEP_PROVER: OnceCell<Arc<Prover<Fp>>> = OnceCell::new();
static BLOCK_STEP_PROVER: OnceCell<Arc<Prover<Fp>>> = OnceCell::new();
static BLOCK_WRAP_PROVER: OnceCell<Arc<Prover<Fq>>> = OnceCell::new();
static ZKAPP_STEP_OPT_SIGNED_OPT_SIGNED_PROVER: OnceCell<Arc<Prover<Fp>>> = OnceCell::new();
static ZKAPP_STEP_OPT_SIGNED_PROVER: OnceCell<Arc<Prover<Fp>>> = OnceCell::new();
static ZKAPP_STEP_PROOF_PROVER: OnceCell<Arc<Prover<Fp>>> = OnceCell::new();

fn default_circuits_config() -> &'static CircuitsConfig {
    openmina_core::NetworkConfig::global().circuits_config
}

async fn get_or_make_tx_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
    get_or_make::<StepTransactionProof, _>(&TX_STEP_PROVER, None, config.step_transaction_gates)
        .await
}
async fn get_or_make_tx_wrap_prover(
    config: &CircuitsConfig,
    verifier_index: Option<TransactionVerifier>,
) -> Arc<Prover<Fq>> {
    get_or_make::<WrapTransactionProof, _>(
        &TX_WRAP_PROVER,
        verifier_index.map(Into::into),
        config.wrap_transaction_gates,
    )
    .await
}
async fn get_or_make_merge_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
    get_or_make::<StepMergeProof, _>(&MERGE_STEP_PROVER, None, config.step_merge_gates).await
}
async fn get_or_make_block_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
    get_or_make::<StepBlockProof, _>(&BLOCK_STEP_PROVER, None, config.step_blockchain_gates).await
}
async fn get_or_make_block_wrap_prover(
    config: &CircuitsConfig,
    verifier_index: Option<BlockVerifier>,
) -> Arc<Prover<Fq>> {
    get_or_make::<WrapBlockProof, _>(
        &BLOCK_WRAP_PROVER,
        verifier_index.map(Into::into),
        config.wrap_blockchain_gates,
    )
    .await
}
async fn get_or_make_zkapp_step_opt_signed_opt_signed_prover(
    config: &CircuitsConfig,
) -> Arc<Prover<Fp>> {
    get_or_make::<StepZkappOptSignedOptSignedProof, _>(
        &ZKAPP_STEP_OPT_SIGNED_OPT_SIGNED_PROVER,
        None,
        config.step_transaction_opt_signed_opt_signed_gates,
    )
    .await
}
async fn get_or_make_zkapp_step_opt_signed_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
    get_or_make::<StepZkappOptSignedProof, _>(
        &ZKAPP_STEP_OPT_SIGNED_PROVER,
        None,
        config.step_transaction_opt_signed_gates,
    )
    .await
}
async fn get_or_make_zkapp_step_proof_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
    get_or_make::<StepZkappProvedProof, _>(
        &ZKAPP_STEP_PROOF_PROVER,
        None,
        config.step_transaction_proved_gates,
    )
    .await
}

mod prover_makers {
    use super::*;

    impl BlockProver {
        pub async fn make_async(
            block_verifier_index: Option<BlockVerifier>,
            tx_verifier_index: Option<TransactionVerifier>,
        ) -> Self {
            let config = default_circuits_config();
            let block_step_prover = get_or_make_block_step_prover(config).await;
            let block_wrap_prover =
                get_or_make_block_wrap_prover(config, block_verifier_index).await;
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config, tx_verifier_index).await;

            Self {
                block_step_prover,
                block_wrap_prover,
                tx_wrap_prover,
            }
        }

        #[cfg(not(target_family = "wasm"))]
        pub fn make(
            block_verifier_index: Option<BlockVerifier>,
            tx_verifier_index: Option<TransactionVerifier>,
        ) -> Self {
            block_on(Self::make_async(block_verifier_index, tx_verifier_index))
        }
    }

    impl TransactionProver {
        pub async fn make_async(tx_verifier_index: Option<TransactionVerifier>) -> Self {
            let config = default_circuits_config();
            let tx_step_prover = get_or_make_tx_step_prover(config).await;
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config, tx_verifier_index).await;
            let merge_step_prover = get_or_make_merge_step_prover(config).await;

            Self {
                tx_step_prover,
                tx_wrap_prover,
                merge_step_prover,
            }
        }

        #[cfg(not(target_family = "wasm"))]
        pub fn make(tx_verifier_index: Option<TransactionVerifier>) -> Self {
            block_on(Self::make_async(tx_verifier_index))
        }
    }

    impl ZkappProver {
        pub async fn make_async(tx_verifier_index: Option<TransactionVerifier>) -> Self {
            let config = default_circuits_config();
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config, tx_verifier_index).await;
            let merge_step_prover = get_or_make_merge_step_prover(config).await;
            let step_opt_signed_opt_signed_prover =
                get_or_make_zkapp_step_opt_signed_opt_signed_prover(config).await;
            let step_opt_signed_prover = get_or_make_zkapp_step_opt_signed_prover(config).await;
            let step_proof_prover = get_or_make_zkapp_step_proof_prover(config).await;

            Self {
                tx_wrap_prover,
                merge_step_prover,
                step_opt_signed_opt_signed_prover,
                step_opt_signed_prover,
                step_proof_prover,
            }
        }

        #[cfg(not(target_family = "wasm"))]
        pub fn make(tx_verifier_index: Option<TransactionVerifier>) -> Self {
            block_on(Self::make_async(tx_verifier_index))
        }
    }
}

#[derive(Clone)]
pub struct BlockProver {
    pub block_step_prover: Arc<Prover<Fp>>,
    pub block_wrap_prover: Arc<Prover<Fq>>,
    pub tx_wrap_prover: Arc<Prover<Fq>>,
}

#[derive(Clone)]
pub struct TransactionProver {
    pub tx_step_prover: Arc<Prover<Fp>>,
    pub tx_wrap_prover: Arc<Prover<Fq>>,
    pub merge_step_prover: Arc<Prover<Fp>>,
}

#[derive(Clone)]
pub struct ZkappProver {
    pub tx_wrap_prover: Arc<Prover<Fq>>,
    pub merge_step_prover: Arc<Prover<Fp>>,
    pub step_opt_signed_opt_signed_prover: Arc<Prover<Fp>>,
    pub step_opt_signed_prover: Arc<Prover<Fp>>,
    pub step_proof_prover: Arc<Prover<Fp>>,
}

fn decode_constraints_data<F: FieldWitness>(
    mut internal_vars: &[u8],
    mut rows_rev: &[u8],
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

    let internal_vars: InternalVarsRaw = HashMap::binprot_read(&mut internal_vars).unwrap();
    let rows_rev: Vec<Vec<Option<VRaw>>> = Vec::binprot_read(&mut rows_rev).unwrap();

    let internal_vars: InternalVars<F> = internal_vars
        .into_iter()
        .map(|(id, (list, opt))| {
            let id = id as usize;
            let list: Vec<_> = list
                .iter()
                .map(|(n, v)| (n.to_field::<F>().unwrap(), V::from(v)))
                .collect();
            let opt = opt.as_ref().map(|v| BigInt::to_field::<F>(v).unwrap());
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

async fn read_constraints_data<F: FieldWitness>(
    internal_vars_path: &Path,
    rows_rev_path: &Path,
) -> Option<(InternalVars<F>, Vec<Vec<Option<V>>>)> {
    // ((Fp.t * V.t) list * Fp.t option)
    let internal_vars = circuit_blobs::fetch(&internal_vars_path).await.ok()?;

    // V.t option array list
    let rows_rev = circuit_blobs::fetch(&rows_rev_path).await.ok()?;
    // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rows_rev.bin")).unwrap();

    decode_constraints_data(internal_vars.as_ref(), rows_rev.as_ref())
}
