use node::core::channels::mpsc;
use node::core::thread;
use node::p2p::service_impl::TaskSpawner;

#[derive(Clone)]
pub struct P2pTaskSpawner {
    shutdown: mpsc::Sender<()>,
}

impl P2pTaskSpawner {
    pub fn new(shutdown: mpsc::Sender<()>) -> Self {
        Self { shutdown }
    }
}

impl TaskSpawner for P2pTaskSpawner {
    fn spawn_main<F>(&self, name: &str, fut: F)
    where
        F: 'static + Send + std::future::Future<Output = ()>,
    {
        let shutdown = self.shutdown.clone();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        thread::Builder::new()
            .name(format!("openmina_p2p_{name}"))
            .spawn(move || {
                let fut = async {
                    tokio::select! {
                        _ = shutdown.closed() => {}
                        _ = fut => {}
                    }
                };
                let local_set = tokio::task::LocalSet::new();
                local_set.block_on(&runtime, fut);
            })
            .unwrap();
    }
}
