use std::{
    task::Poll,
    time::{Duration, Instant},
};

use futures::Stream;
use pin_project_lite::pin_project;

use crate::cluster::{TimestampEvent, TimestampSource};

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
    fn map_errors<F>(self, is_error: F) -> MapErrors<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        MapErrors {
            stream: self,
            is_error,
        }
    }
}

macro_rules! cluster_stream_impls {
    ($name:ident < $S:ident > ) => {
        cluster_stream_impls!($name<$S,>);
    };
    ($name:ident < $S:ident, $( $($param:ident),+ $(,)? )? >) => {
        impl<$S, $($($param),* )?> std::ops::Deref for $name<$S, $($($param),* )?> {
            type Target = $S;

            fn deref(&self) -> &Self::Target {
                &self.stream
            }
        }

        impl<$S, $($($param),* )?> std::ops::DerefMut for $name<$S, $($($param),* )?> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.stream
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
    pub struct MapErrors<S, F> {
        #[pin]
        stream: S,
        is_error: F,
    }
}

impl<S, F> Stream for MapErrors<S, F>
where
    S: Stream,
    F: FnMut(&S::Item) -> bool,
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

cluster_stream_impls!(MapErrors<S, F>);
