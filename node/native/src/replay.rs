use std::cell::RefCell;

use node::{
    core::channels::mpsc, ledger::LedgerManager, recorder::StateWithInputActionsReader,
    service::Recorder, snark::VerifierKind, ActionWithMeta, BuildEnv, Store,
};
use rand::{rngs::StdRng, SeedableRng};

use crate::{rpc::RpcService, NodeService, ReplayerState};

pub fn replay_state_with_input_actions(
    dir: &str,
    dynamic_effects_lib: Option<String>,
    mut check_build_env: impl FnMut(&BuildEnv, &BuildEnv) -> anyhow::Result<()>,
) -> anyhow::Result<::node::Node<NodeService>> {
    eprintln!("replaying node based on initial state and actions from the dir: {dir}");
    let reader = StateWithInputActionsReader::new(dir);

    eprintln!(
        "reading initial state from file: {}",
        reader.initial_state_path().as_path().to_str().unwrap()
    );
    let initial_state = match reader.read_initial_state() {
        Err(err) => anyhow::bail!("failed to read initial state. err: {err}"),
        Ok(v) => v,
    };

    let state = {
        let mut state = initial_state.state.into_owned();
        // TODO(binier): we shouldn't have to do this, but serialized
        // index/srs doesn't match deserialized one.
        state.snark.block_verify.verifier_index =
            node::snark::get_verifier_index(VerifierKind::Blockchain).into();
        state.snark.block_verify.verifier_srs = node::snark::get_srs();
        state
    };

    let effects: node::Effects<NodeService> = dynamic_effects_lib
        .as_ref()
        .map_or(replayer_effects, |_| replayer_effects_with_dyn_effects);
    let p2p_sec_key = initial_state.p2p_sec_key;

    let service = NodeService {
        rng: StdRng::seed_from_u64(initial_state.rng_seed),
        event_sender: mpsc::unbounded_channel().0,
        event_receiver: mpsc::unbounded_channel().1.into(),
        cmd_sender: mpsc::unbounded_channel().0,
        ledger_manager: LedgerManager::spawn(Default::default()),
        peers: Default::default(),
        #[cfg(feature = "p2p-libp2p")]
        mio: node::p2p::service_impl::mio::MioService::mocked(),
        network: Default::default(),
        block_producer: None,
        keypair: p2p_sec_key.into(),
        rpc: RpcService::new(),
        snark_worker_sender: None,
        stats: Default::default(),
        recorder: Recorder::None,
        replayer: Some(ReplayerState {
            initial_monotonic: redux::Instant::now(),
            initial_time: state.time(),
            expected_actions: Default::default(),
            replay_dynamic_effects_lib: dynamic_effects_lib.unwrap_or_default(),
        }),
        invariants_state: Default::default(),
    };

    let mut node = ::node::Node::new(state, service, Some(effects));

    let store = node.store_mut();

    let replay_env = BuildEnv::get();
    check_build_env(&store.state().config.build, &replay_env)?;

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
            assert_eq!(
                expected_actions.len(),
                0,
                "not all expected effects of the input action were dispatched! Ones left: {expected_actions:?}"
            );
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
            node::replay_status::set_next_force_enabled();
            store.dispatch(action);
        }
    }
    Ok(node)
}

fn replayer_effects_with_dyn_effects(store: &mut Store<NodeService>, action: ActionWithMeta) {
    dyn_effects(store, &action);
    replayer_effects(store, action);
}

fn replayer_effects(store: &mut Store<NodeService>, action: ActionWithMeta) {
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
    static DYN_EFFECTS_LIB: RefCell<Option<DynEffectsLib>> = const { RefCell::new(None)};
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
