use rand::rngs::StdRng;
use rand::SeedableRng;
use snarker::p2p::service_impl::libp2p::Libp2pService;
use snarker::recorder::{Recorder, StateWithInputActionsReader};
use snarker::snark::VerifierKind;
use snarker::{ActionWithMeta, Store};
use tokio::sync::mpsc;

use crate::commands::snarker::{ReplayerState, RpcService, SnarkerService};

#[derive(Debug, clap::Args)]
/// Replay node using initial state and input actions.
pub struct ReplayStateWithInputActions {
    #[arg(long, short, default_value = "~/.openmina/recorder")]
    pub dir: String,

    /// Verbosity level
    #[arg(long, short, default_value = "info")]
    pub verbosity: tracing::Level,
}

impl ReplayStateWithInputActions {
    pub fn run(self) -> Result<(), crate::CommandError> {
        crate::commands::snarker::tracing::initialize(self.verbosity);

        eprintln!(
            "replaying node based on initial state and actions from the dir: {}",
            self.dir
        );
        let dir = shellexpand::full(&self.dir)?.into_owned();
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
                snarker::snark::get_verifier_index(VerifierKind::Blockchain).into();
            state.snark.block_verify.verifier_srs = snarker::snark::get_srs().into();
            state
        };

        let service = SnarkerService {
            rng: StdRng::seed_from_u64(initial_state.rng_seed),
            event_sender: mpsc::unbounded_channel().0,
            p2p_event_sender: mpsc::unbounded_channel().0,
            event_receiver: mpsc::unbounded_channel().1.into(),
            cmd_sender: mpsc::unbounded_channel().0,
            ledger: Default::default(),
            peers: Default::default(),
            libp2p: Libp2pService::mocked().0,
            rpc: RpcService::new(),
            snark_worker_sender: None,
            stats: Default::default(),
            recorder: Recorder::None,
            replayer: Some(ReplayerState {
                initial_monotonic: redux::Instant::now(),
                initial_time: state.time(),
                expected_actions: Default::default(),
            }),
        };

        let mut snarker = ::snarker::Snarker::new(state, service, Some(replayer_effects));
        let store = snarker.store_mut();

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

            if input_action.is_none() {
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
                continue;
            }

            let is_done = if action.action.is_none() {
                let action = actions.next().unwrap();
                expected_actions.push_back((action.kind, action.meta));
                false
            } else {
                true
            };

            if is_done || actions.peek().is_none() {
                if !is_done {
                    eprintln!("Warning! Executing action for which we might not have all effect actions recorded.");
                }
                let action = input_action.take().unwrap();
                store.dispatch(action);
            }
        }

        Ok(())
    }
}

fn replayer_effects(store: &mut Store<SnarkerService>, action: ActionWithMeta) {
    let replayer = store.service.replayer.as_mut().unwrap();
    let (kind, meta) = match replayer.expected_actions.pop_front() {
        Some(v) => v,
        None => panic!("unexpected action: {:?}", action),
    };

    assert_eq!(kind, action.action().kind());
    assert_eq!(meta.time(), action.meta().time());

    snarker::effects(store, action)
}
