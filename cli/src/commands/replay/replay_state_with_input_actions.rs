use std::cell::RefCell;

use libp2p_identity::Keypair;
use node::core::channels::mpsc;
use node::recorder::{Recorder, StateWithInputActionsReader};
use node::snark::VerifierKind;
use node::{ActionWithMeta, BuildEnv, Store};
use openmina_node_native::{rpc::RpcService, NodeService, ReplayerState};
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Debug, clap::Args)]
/// Replay node using initial state and input actions.
pub struct ReplayStateWithInputActions {
    #[arg(long, short, default_value = "~/.openmina/recorder")]
    pub dir: String,

    #[arg(long, default_value = "./target/release/libreplay_dynamic_effects.so")]
    pub dynamic_effects_lib: String,

    /// Verbosity level
    #[arg(long, short, default_value = "info")]
    pub verbosity: tracing::Level,
}

impl ReplayStateWithInputActions {
    pub fn run(self) -> Result<(), crate::CommandError> {
        openmina_node_native::tracing::initialize(self.verbosity);

        eprintln!(
            "replaying node based on initial state and actions from the dir: {}",
            self.dir
        );
        let dir = shellexpand::full(&self.dir)?.into_owned();
        let dynamic_effects_lib = shellexpand::full(&self.dynamic_effects_lib)?.into_owned();
        let reader = StateWithInputActionsReader::new(&dir);

        eprintln!(
            "reading initial state from file: {}",
            reader.initial_state_path().as_path().to_str().unwrap()
        );
        let initial_state = reader.read_initial_state().unwrap();

        let state = {
            let mut state = initial_state.state.into_owned();
            // TODO(binier): we shouldn't have to do this, but serialized
            // index/srs doesn't match deserialized one.
            state.snark.block_verify.verifier_index =
                node::snark::get_verifier_index(VerifierKind::Blockchain).into();
            state.snark.block_verify.verifier_srs = node::snark::get_srs().into();
            state
        };

        let service = NodeService {
            rng: StdRng::seed_from_u64(initial_state.rng_seed),
            event_sender: mpsc::unbounded_channel().0,
            event_receiver: mpsc::unbounded_channel().1.into(),
            cmd_sender: mpsc::unbounded_channel().0,
            ledger: Default::default(),
            peers: Default::default(),
            #[cfg(feature = "p2p-libp2p")]
            libp2p: node::p2p::service_impl::libp2p::Libp2pService::mocked().0,
            #[cfg(not(feature = "p2p-libp2p"))]
            mio: node::p2p::service_impl::mio::MioService::mocked(),
            keypair: Keypair::generate_ed25519(),
            rpc: RpcService::new(),
            snark_worker_sender: None,
            stats: Default::default(),
            recorder: Recorder::None,
            replayer: Some(ReplayerState {
                initial_monotonic: redux::Instant::now(),
                initial_time: state.time(),
                expected_actions: Default::default(),
                replay_dynamic_effects_lib: dynamic_effects_lib,
            }),
            invariants_state: Default::default(),
        };

        let mut node = ::node::Node::new(state, service, Some(replayer_effects));
        let store = node.store_mut();

        let replay_env = BuildEnv::get();
        check_env(&store.state().config.build, &replay_env);

        eprintln!("reading actions from dir: {dir}");

        let mut input_action = None;
        let mut actions = reader
            .read_actions()
            .flat_map(|(path, actions)| {
                let file_path = path.as_path().to_str().unwrap();
                eprintln!("processing actions from file: {file_path}");
                actions
            })
            .peekable();

        while let Some(action) = actions.peek() {
            let replayer = store.service.replayer.as_mut().unwrap();
            let expected_actions = &mut replayer.expected_actions;

            let action = if input_action.is_none() {
                expected_actions.clear();
                let (action, meta) = actions
                    .next()
                    .unwrap()
                    .as_action_with_meta()
                    .expect("expected input action, got effect action")
                    .split();
                let kind = action.kind();
                let _ = input_action.insert(action);
                expected_actions.push_back((kind, meta));
                actions.peek()
            } else {
                Some(action)
            };

            let is_done = if let Some(action) = action {
                if action.action.is_none() {
                    let action = actions.next().unwrap();
                    expected_actions.push_back((action.kind, action.meta));
                    false
                } else {
                    true
                }
            } else {
                false
            };

            if is_done || actions.peek().is_none() {
                if !is_done {
                    eprintln!("Warning! Executing last action for which we might not have all effect actions recorded.");
                }
                let action = input_action.take().unwrap();
                store.dispatch(action);
            }
        }

        Ok(())
    }
}

fn replayer_effects(store: &mut Store<NodeService>, action: ActionWithMeta) {
    dyn_effects(store, &action);

    let replayer = store.service.replayer.as_mut().unwrap();
    let (kind, meta) = match replayer.expected_actions.pop_front() {
        Some(v) => v,
        None => panic!("unexpected action: {:?}", action),
    };

    assert_eq!(kind, action.action().kind());
    assert_eq!(meta.time(), action.meta().time());

    node::effects(store, action)
}

fn dyn_effects(store: &mut Store<NodeService>, action: &ActionWithMeta) {
    DYN_EFFECTS_LIB.with(move |cell| loop {
        let mut opt = cell.borrow_mut();
        let fun = match opt.as_ref() {
            None => {
                let lib_path = &store
                    .service
                    .replayer
                    .as_ref()
                    .unwrap()
                    .replay_dynamic_effects_lib;
                opt.insert(DynEffectsLib::load(lib_path)).fun
            }
            Some(lib) => lib.fun,
        };

        match fun(store, action) {
            0 => return,
            1 => {
                opt.take();
                let lib_path = &store
                    .service
                    .replayer
                    .as_ref()
                    .unwrap()
                    .replay_dynamic_effects_lib;
                let query_modified = || match std::fs::metadata(lib_path).and_then(|v| v.modified())
                {
                    Err(err) => {
                        eprintln!("Error querying replay_dynamic_effects_lib modified time: {err}");
                        redux::SystemTime::UNIX_EPOCH
                    }
                    Ok(v) => v,
                };

                let initial_time = query_modified();
                let sleep_dur = std::time::Duration::from_millis(100);
                eprintln!("Waiting for {lib_path} to be modified.");
                while initial_time >= query_modified() {
                    std::thread::sleep(sleep_dur);
                }
            }
            ret => panic!("unknown `replay_dynamic_effects` return number: {ret}"),
        }
    });
}

thread_local! {
    static DYN_EFFECTS_LIB: RefCell<Option<DynEffectsLib>> = RefCell::new(None);
}

struct DynEffectsLib {
    handle: *mut nix::libc::c_void,
    fun: fn(&mut Store<NodeService>, &ActionWithMeta) -> u8,
}

impl DynEffectsLib {
    fn load(lib_path: &str) -> Self {
        use nix::libc::{c_void, dlopen, dlsym, RTLD_NOW};
        use std::ffi::CString;

        let filename = CString::new(lib_path).unwrap();

        let handle = unsafe { dlopen(filename.as_ptr(), RTLD_NOW) };
        if handle.is_null() {
            panic!("Failed to resolve dlopen {lib_path}")
        }

        let fun_name = CString::new("replay_dynamic_effects").unwrap();
        let fun = unsafe { dlsym(handle, fun_name.as_ptr()) };
        if fun.is_null() {
            panic!("Failed to resolve '{}'", &fun_name.to_str().unwrap());
        }

        Self {
            handle,
            fun: unsafe { std::mem::transmute::<*mut c_void, _>(fun) },
        }
    }
}

impl Drop for DynEffectsLib {
    fn drop(&mut self) {
        let ret = unsafe { nix::libc::dlclose(self.handle) };
        if ret != 0 {
            panic!("Error while closing lib");
        }
    }
}

pub fn check_env(record_env: &BuildEnv, replay_env: &BuildEnv) {
    let is_git_same = record_env.git.commit_hash == replay_env.git.commit_hash;
    let is_cargo_same = record_env.cargo == replay_env.cargo;
    let is_rustc_same = record_env.rustc == replay_env.rustc;

    if !is_git_same {
        let diff = format!(
            "recorded:\n{:?}\n\ncurrent:\n{:?}",
            record_env.git, replay_env.git
        );
        let msg = format!("git build env mismatch!\n{diff}");
        if console::user_attended() {
            use dialoguer::Confirm;

            let prompt = format!("{msg}\nDo you want to continue?");
            if Confirm::new().with_prompt(prompt).interact().unwrap() {
            } else {
                std::process::exit(1);
            }
        } else {
            std::process::exit(1);
        }
    }

    if !is_cargo_same {
        let diff = format!(
            "recorded:\n{:?}\n\ncurrent:\n{:?}",
            record_env.cargo, replay_env.cargo
        );
        let msg = format!("cargo build env mismatch!\n{diff}");
        eprintln!("{msg}");
    }

    if !is_rustc_same {
        let diff = format!(
            "recorded:\n{:?}\n\ncurrent:\n{:?}",
            record_env.rustc, replay_env.rustc
        );
        let msg = format!("rustc build env mismatch!\n{diff}");
        eprintln!("{msg}");
    }
}
