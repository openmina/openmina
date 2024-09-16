use std::{collections::HashMap, path::Path, sync::Arc};

use kimchi::circuits::gate::CircuitGate;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use once_cell::sync::OnceCell;
use openmina_core::network::CircuitsConfig;

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

pub fn devnet_circuit_directory() -> &'static str {
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

#[derive(Clone)]
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

#[cfg(not(target_family = "wasm"))]
fn read_gates_file<F: FieldWitness>(
    filepath: &impl AsRef<Path>,
) -> std::io::Result<Vec<CircuitGate<F>>> {
    let file = std::fs::File::open(filepath)?;
    let reader = std::io::BufReader::new(file);
    decode_gates_file(reader)
}

#[cfg(target_family = "wasm")]
mod http {
    use openmina_core::thread;
    use wasm_bindgen::prelude::*;

    fn to_io_err(err: JsValue) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{err:?}"))
    }

    async fn _get_bytes(url: String) -> std::io::Result<Vec<u8>> {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::Response;

        // let window = js_sys::global().dyn_into::<web_sys::WorkerGlobalScope>().unwrap();
        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_str(&url))
            .await
            .map_err(to_io_err)?;

        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();
        let js = JsFuture::from(resp.array_buffer().map_err(to_io_err)?)
            .await
            .map_err(to_io_err)?;
        Ok(js_sys::Uint8Array::new(&js).to_vec())
    }

    pub async fn get_bytes(url: &str) -> std::io::Result<Vec<u8>> {
        let url = url.to_owned();
        if thread::is_web_worker_thread() {
            thread::run_async_fn_in_main_thread(move || _get_bytes(url)).await.expect("failed to run task in the main thread! Maybe main thread crashed or not initialized?")
        } else {
            _get_bytes(url).await
        }
    }
}

#[cfg(target_family = "wasm")]
async fn read_gates_file<F: FieldWitness>(
    filepath: impl AsRef<Path>,
) -> std::io::Result<Vec<CircuitGate<F>>> {
    let url = filepath.as_ref().to_str().unwrap();
    let resp = http::get_bytes(url).await?;
    decode_gates_file(&mut resp.as_slice())
}

#[cfg(not(target_family = "wasm"))]
fn make_gates<F: FieldWitness>(
    filename: &str,
) -> (
    HashMap<usize, (Vec<(F, V)>, Option<F>)>,
    Vec<Vec<Option<V>>>,
    Vec<CircuitGate<F>>,
) {
    let circuits_config = openmina_core::NetworkConfig::global().circuits_config;
    let base_dir = std::env::var("OPENMINA_CIRCUIT_BLOBS_BASE_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    let base_dir = Path::new(&base_dir);
    let base_dir = if base_dir.exists() {
        base_dir
    } else {
        Path::new("/usr/local/lib/openmina/circuit-blobs")
    };
    let base_dir = base_dir.join(circuits_config.directory_name);

    let internal_vars_path = base_dir.join(format!("{}_internal_vars.bin", filename));
    let rows_rev_path = base_dir.join(format!("{}_rows_rev.bin", filename));
    let gates_path = base_dir.join(format!("{}_gates.json", filename));

    let gates: Vec<CircuitGate<F>> = read_gates_file(&gates_path).unwrap();
    let (internal_vars_path, rows_rev_path) =
        read_constraints_data::<F>(&internal_vars_path, &rows_rev_path).unwrap();

    (internal_vars_path, rows_rev_path, gates)
}

#[cfg(target_family = "wasm")]
async fn make_gates<F: FieldWitness>(
    filename: &str,
) -> (
    HashMap<usize, (Vec<(F, V)>, Option<F>)>,
    Vec<Vec<Option<V>>>,
    Vec<CircuitGate<F>>,
) {
    let circuits_config = openmina_core::NetworkConfig::global().circuits_config;
    let base_dir = Path::new("/circuit-blobs");
    let base_dir = base_dir.join(circuits_config.directory_name);

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

macro_rules! get_or_make {
    ($constant: ident, $type: ty, $filename: expr) => {{
        if let Some(prover) = $constant.get() {
            return prover.clone();
        }

        let (internal_vars, rows_rev, gates) = {
            #[cfg(not(target_family = "wasm"))]
            let res = make_gates($filename);
            #[cfg(target_family = "wasm")]
            let res = make_gates($filename).await;
            res
        };

        let index = make_prover_index::<$type, _>(gates);
        let prover = Prover {
            internal_vars,
            rows_rev,
            index,
        };

        $constant.get_or_init(|| prover.into()).clone()
    }};
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

#[cfg(not(target_family = "wasm"))]
mod prover_makers {
    use super::*;

    fn get_or_make_tx_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(
            TX_STEP_PROVER,
            StepTransactionProof,
            config.step_transaction_gates
        )
    }
    fn get_or_make_tx_wrap_prover(config: &CircuitsConfig) -> Arc<Prover<Fq>> {
        get_or_make!(
            TX_WRAP_PROVER,
            WrapTransactionProof,
            config.wrap_transaction_gates
        )
    }
    fn get_or_make_merge_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(MERGE_STEP_PROVER, StepMergeProof, config.step_merge_gates)
    }
    fn get_or_make_block_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(
            BLOCK_STEP_PROVER,
            StepBlockProof,
            config.step_blockchain_gates
        )
    }
    fn get_or_make_block_wrap_prover(config: &CircuitsConfig) -> Arc<Prover<Fq>> {
        get_or_make!(
            BLOCK_WRAP_PROVER,
            WrapBlockProof,
            config.wrap_blockchain_gates
        )
    }
    fn get_or_make_zkapp_step_opt_signed_opt_signed_prover(
        config: &CircuitsConfig,
    ) -> Arc<Prover<Fp>> {
        get_or_make!(
            ZKAPP_STEP_OPT_SIGNED_OPT_SIGNED_PROVER,
            StepZkappOptSignedOptSignedProof,
            config.step_transaction_opt_signed_opt_signed_gates
        )
    }
    fn get_or_make_zkapp_step_opt_signed_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(
            ZKAPP_STEP_OPT_SIGNED_PROVER,
            StepZkappOptSignedProof,
            config.step_transaction_opt_signed_gates
        )
    }
    fn get_or_make_zkapp_step_proof_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(
            ZKAPP_STEP_PROOF_PROVER,
            StepZkappProvedProof,
            config.step_transaction_proved_gates
        )
    }

    impl BlockProver {
        pub fn make() -> Self {
            let config = default_circuits_config();
            let block_step_prover = get_or_make_block_step_prover(config);
            let block_wrap_prover = get_or_make_block_wrap_prover(config);
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config);

            Self {
                block_step_prover,
                block_wrap_prover,
                tx_wrap_prover,
            }
        }
    }

    impl TransactionProver {
        pub fn make() -> Self {
            let config = default_circuits_config();
            let tx_step_prover = get_or_make_tx_step_prover(config);
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config);
            let merge_step_prover = get_or_make_merge_step_prover(config);

            Self {
                tx_step_prover,
                tx_wrap_prover,
                merge_step_prover,
            }
        }
    }

    impl ZkappProver {
        pub fn make() -> Self {
            let config = default_circuits_config();
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config);
            let merge_step_prover = get_or_make_merge_step_prover(config);
            let step_opt_signed_opt_signed_prover =
                get_or_make_zkapp_step_opt_signed_opt_signed_prover(config);
            let step_opt_signed_prover = get_or_make_zkapp_step_opt_signed_prover(config);
            let step_proof_prover = get_or_make_zkapp_step_proof_prover(config);

            Self {
                tx_wrap_prover,
                merge_step_prover,
                step_opt_signed_opt_signed_prover,
                step_opt_signed_prover,
                step_proof_prover,
            }
        }
    }
}

#[cfg(target_family = "wasm")]
mod prover_makers {
    use super::*;

    async fn get_or_make_tx_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(
            TX_STEP_PROVER,
            StepTransactionProof,
            config.step_transaction_gates
        )
    }
    async fn get_or_make_tx_wrap_prover(config: &CircuitsConfig) -> Arc<Prover<Fq>> {
        get_or_make!(
            TX_WRAP_PROVER,
            WrapTransactionProof,
            config.wrap_transaction_gates
        )
    }
    async fn get_or_make_merge_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(MERGE_STEP_PROVER, StepMergeProof, config.step_merge_gates)
    }
    async fn get_or_make_block_step_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(
            BLOCK_STEP_PROVER,
            StepBlockProof,
            config.step_blockchain_gates
        )
    }
    async fn get_or_make_block_wrap_prover(config: &CircuitsConfig) -> Arc<Prover<Fq>> {
        get_or_make!(
            BLOCK_WRAP_PROVER,
            WrapBlockProof,
            config.wrap_blockchain_gates
        )
    }
    async fn get_or_make_zkapp_step_opt_signed_opt_signed_prover(
        config: &CircuitsConfig,
    ) -> Arc<Prover<Fp>> {
        get_or_make!(
            ZKAPP_STEP_OPT_SIGNED_OPT_SIGNED_PROVER,
            StepZkappOptSignedOptSignedProof,
            config.step_transaction_opt_signed_opt_signed_gates
        )
    }
    async fn get_or_make_zkapp_step_opt_signed_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(
            ZKAPP_STEP_OPT_SIGNED_PROVER,
            StepZkappOptSignedProof,
            config.step_transaction_opt_signed_gates
        )
    }
    async fn get_or_make_zkapp_step_proof_prover(config: &CircuitsConfig) -> Arc<Prover<Fp>> {
        get_or_make!(
            ZKAPP_STEP_PROOF_PROVER,
            StepZkappProvedProof,
            config.step_transaction_proved_gates
        )
    }

    impl BlockProver {
        pub async fn make() -> Self {
            let config = default_circuits_config();
            let block_step_prover = get_or_make_block_step_prover(config).await;
            let block_wrap_prover = get_or_make_block_wrap_prover(config).await;
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config).await;

            Self {
                block_step_prover,
                block_wrap_prover,
                tx_wrap_prover,
            }
        }
    }

    impl TransactionProver {
        pub async fn make() -> Self {
            let config = default_circuits_config();
            let tx_step_prover = get_or_make_tx_step_prover(config).await;
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config).await;
            let merge_step_prover = get_or_make_merge_step_prover(config).await;

            Self {
                tx_step_prover,
                tx_wrap_prover,
                merge_step_prover,
            }
        }
    }

    impl ZkappProver {
        pub async fn make() -> Self {
            let config = default_circuits_config();
            let tx_wrap_prover = get_or_make_tx_wrap_prover(config).await;
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

#[cfg(not(target_family = "wasm"))]
fn read_constraints_data<F: FieldWitness>(
    internal_vars_path: &Path,
    rows_rev_path: &Path,
) -> Option<(InternalVars<F>, Vec<Vec<Option<V>>>)> {
    // ((Fp.t * V.t) list * Fp.t option)
    let Ok(internal_vars) = std::fs::read(internal_vars_path)
    // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("internal_vars.bin"))
    else {
        return None;
    };

    // V.t option array list
    let rows_rev = std::fs::read(rows_rev_path).unwrap();
    // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rows_rev.bin")).unwrap();

    decode_constraints_data(internal_vars.as_slice(), rows_rev.as_slice())
}

#[cfg(target_family = "wasm")]
async fn read_constraints_data<F: FieldWitness>(
    internal_vars_path: &Path,
    rows_rev_path: &Path,
) -> Option<(InternalVars<F>, Vec<Vec<Option<V>>>)> {
    // ((Fp.t * V.t) list * Fp.t option)
    let internal_vars = http::get_bytes(internal_vars_path.to_str().unwrap())
        .await
        .ok()?;

    // V.t option array list
    let rows_rev = http::get_bytes(rows_rev_path.to_str().unwrap())
        .await
        .ok()?;
    // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rows_rev.bin")).unwrap();

    decode_constraints_data(internal_vars.as_ref(), rows_rev.as_ref())
}
