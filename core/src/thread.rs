#[cfg(not(target_arch = "wasm32"))]
pub use std::thread::*;
#[cfg(target_arch = "wasm32")]
pub use wasm_thread::*;

#[cfg(target_family = "wasm")]
mod main_thread {
    use crate::channels::{mpsc, oneshot};
    use std::{future::Future, pin::Pin};

    pub type TaskForMainThread = Pin<Box<dyn 'static + Send + Future<Output = ()>>>;

    static MAIN_THREAD_TASK_SENDER: once_cell::sync::OnceCell<
        mpsc::UnboundedSender<TaskForMainThread>,
    > = once_cell::sync::OnceCell::new();

    pub fn main_thread_init() {
        assert!(
            !super::is_web_worker_thread(),
            "Must be called in the main thread!"
        );

        MAIN_THREAD_TASK_SENDER.get_or_init(|| {
            let (task_sender, mut task_receiver) = mpsc::unbounded_channel();
            wasm_bindgen_futures::spawn_local(async move {
                while let Some(task) = task_receiver.recv().await {
                    wasm_bindgen_futures::spawn_local(task);
                }
            });
            task_sender
        });
    }

    pub fn start_task_in_main_thread<F>(task: F)
    where
        F: 'static + Send + Future<Output = ()>,
    {
        let sender = MAIN_THREAD_TASK_SENDER
            .get()
            .expect("main thread not initialized");
        let _ = sender.send(Box::pin(task));
    }

    pub async fn run_task_in_main_thread<F, T>(task: F) -> Option<T>
    where
        T: 'static + Send,
        F: 'static + Send + Future<Output = T>,
    {
        let sender = MAIN_THREAD_TASK_SENDER
            .get()
            .expect("main thread not initialized");
        let (tx, rx) = oneshot::channel();
        let _ = sender.send(Box::pin(async move {
            let _ = tx.send(task.await);
        }));
        rx.await.ok()
    }
}
#[cfg(target_family = "wasm")]
pub use main_thread::*;
