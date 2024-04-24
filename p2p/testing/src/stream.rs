use std::{
    ops::{Deref, DerefMut},
    task::{ready, Poll},
    time::{Duration, Instant},
};

use futures::{Future, Stream, TryStream};
use pin_project_lite::pin_project;

use crate::{
    cluster::{Cluster, ClusterEvent, TimestampEvent, TimestampSource},
    event::RustNodeEvent,
    rust_node::RustNodeId,
};

pub trait ClusterStreamExt: Stream {
    /// Take events during specified period of time.
    fn take_during(self, duration: Duration) -> TakeDuring<Self>
    where
        Self::Item: TimestampEvent,
        Self: TimestampSource + Sized,
    {
        let timeout = self.timestamp() + duration;
        TakeDuring {
            stream: self,
            timeout,
        }
    }

    /// Maps events to ``Result`, according to the `is_error` output.
    fn map_errors(self, is_error: fn(&Self::Item) -> bool) -> MapErrors<Self, Self::Item>
    where
        Self: Sized,
    {
        MapErrors {
            stream: self,
            is_error,
        }
    }

    /// Attempts to execute a predicate over an event stream and evaluate if any
    /// rust node event and state satisfy the predicate.
    fn try_any_with_rust<F>(self, f: F) -> TryAnyWithRustNode<Self, F>
    where
        Self: Sized + TryStream,
        F: FnMut(RustNodeId, RustNodeEvent, &p2p::P2pState) -> bool,
    {
        TryAnyWithRustNode::new(self, f)
    }
}

macro_rules! cluster_stream_impls {
    ($name:ident < $S:ident > ) => {
        cluster_stream_impls!($name<$S,>);
    };
    ($name:ident < $S:ident, $( $($param:ident),+ $(,)? )? >) => {
        impl<$S, $($($param),* )?> Deref for $name<$S, $($($param),* )?> where $S: Deref<Target = Cluster> {
            type Target = Cluster;

            fn deref(&self) -> &Self::Target {
                self.stream.deref()
            }
        }

        impl<$S, $($($param),* )?> DerefMut for $name<$S, $($($param),* )?> where $S: DerefMut<Target = Cluster> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.stream.deref_mut()
            }
        }

        impl<$S, $($($param),* )?> TimestampSource for $name<$S, $($($param),* )?>
        where
            $S: TimestampSource,
        {
            fn timestamp(&self) -> Instant {
                self.stream.timestamp()
            }
        }

        impl<$S, $($($param),* )?> TimestampSource for &$name<$S, $($($param),* )?>
        where
            $S: TimestampSource,
        {
            fn timestamp(&self) -> Instant {
                self.stream.timestamp()
            }
        }

        impl<$S, $($($param),* )?> TimestampSource for &mut $name<$S, $($($param),* )?>
        where
            $S: TimestampSource,
        {
            fn timestamp(&self) -> Instant {
                self.stream.timestamp()
            }
        }
    };
}

impl<T> ClusterStreamExt for T where T: Stream {}

pin_project! {
    pub struct TakeDuring<S> {
        #[pin]
        stream: S,
        timeout: Instant,
    }
}

impl<S> Stream for TakeDuring<S>
where
    S: Stream,
    S::Item: TimestampEvent,
{
    type Item = S::Item;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let poll = this.stream.poll_next(cx);
        if let Poll::Ready(Some(item)) = &poll {
            if let Some(t) = item.timestamp() {
                if t >= *this.timeout {
                    return Poll::Ready(None);
                }
            }
        }
        poll
    }
}

cluster_stream_impls!(TakeDuring<S>);

pin_project! {
    pub struct MapErrors<S, T> {
        #[pin]
        stream: S,
        is_error: fn(&T) -> bool,
    }
}

impl<S, T> Stream for MapErrors<S, T>
where
    S: Stream<Item = T>,
{
    type Item = Result<S::Item, S::Item>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.poll_next(cx).map(|event| {
            event.map(|event| {
                if (this.is_error)(&event) {
                    Err(event)
                } else {
                    Ok(event)
                }
            })
        })
    }
}

cluster_stream_impls!(MapErrors<S, T>);

pin_project! {
    pub struct TryAnyWithCluster<St, F, Fut> {
        #[pin]
        stream: St,
        f: F,
        done: bool,
        #[pin]
        future: Option<Fut>,

    }
}

pin_project! {
    pub struct TryAnyWithRustNode<St, F> {
        #[pin]
        stream: St,
        f: F,
        done: bool,
    }
}

impl<St, F> TryAnyWithRustNode<St, F> {
    pub(crate) fn new(stream: St, f: F) -> Self {
        TryAnyWithRustNode {
            stream,
            f,
            done: false,
        }
    }
}

impl<St, F> Future for TryAnyWithRustNode<St, F>
where
    St: TryStream<Ok = ClusterEvent> + DerefMut<Target = Cluster>,
    F: FnMut(RustNodeId, RustNodeEvent, &p2p::P2pState) -> bool,
{
    type Output = Result<bool, St::Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        Poll::Ready(loop {
            if !*this.done {
                match ready!(this.stream.as_mut().try_poll_next(cx)) {
                    Some(Ok(ClusterEvent::Rust { id, event })) => {
                        if (this.f)(id, event, this.stream.rust_node(id).state()) {
                            *this.done = true;
                            break Ok(true);
                        }
                    }
                    Some(Err(err)) => break Err(err),
                    None => {
                        *this.done = true;
                        break Ok(false);
                    }
                    _ => {}
                }
            } else {
                panic!("TryAnyWithCluster polled after completion")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{future::ready, time::Duration};

    use futures::StreamExt;

    use crate::{
        cluster::{ClusterBuilder, TimestampEvent},
        rust_node::RustNodeConfig,
        stream::ClusterStreamExt,
    };

    #[tokio::test]
    async fn take_during() {
        let mut cluster = ClusterBuilder::new()
            .ports(1000..1002)
            .start()
            .await
            .expect("should build cluster");

        let d = Duration::from_millis(1000);
        let timeout = cluster.timestamp() + d;
        let take_during = cluster.stream().take_during(d);

        let all_under_timeout = take_during
            .all(|event| ready(event.timestamp().map_or(false, |t| t < timeout)))
            .await;
        assert!(all_under_timeout);
    }

    #[tokio::test]
    async fn try_any_with_rust() {
        let mut cluster = ClusterBuilder::new()
            .ports(1010..1012)
            .total_duration(Duration::from_millis(100))
            .start()
            .await
            .expect("should build cluster");

        cluster
            .add_rust_node(RustNodeConfig::default())
            .expect("add node");

        let res = cluster
            .try_stream()
            .try_any_with_rust(|_id, _event, _state: &_| true)
            .await
            .expect("no errors");

        assert!(res);
    }
}
