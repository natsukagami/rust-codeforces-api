use reqwest::Client as HTTP;
/// Client represents a Codeforces API client.
/// It wraps around a reqwest HTTP client and provides rate-limiting.
pub struct Client(rate_limit::Ratelimit<HTTP>);

// Number of requests per second to be rate-limited.
pub const REQUESTS_PER_SECOND: usize = 4;

impl Client {
    /// New creates a new Client.
    pub fn new() -> Self {
        Self(rate_limit::Ratelimit::new(
            HTTP::new(),
            REQUESTS_PER_SECOND,
            std::time::Duration::from_secs(1),
        ))
    }

    /// Borrows and returns the inner HTTP client.
    pub(crate) async fn borrow<'a>(&'a self) -> impl std::ops::Deref<Target = HTTP> + 'a {
        self.0.borrow().await
    }
}

mod rate_limit {
    /// Provides a simple ratelimit lock (that only works in tokio)
    // use tokio::time::
    use std::time::Duration;

    use flume::{bounded as channel, Receiver, Sender};
    use std::ops::Deref;

    /// Holds the underlying `T` in a rate-limited way.
    pub struct Ratelimit<T> {
        inner: T,
        recv: Receiver<()>,
        send: Sender<()>,

        wait_time: Duration,
    }

    struct RatelimitGuard<'a, T> {
        inner: &'a T,
        send: &'a Sender<()>,
        wait_time: &'a Duration,
    }

    impl<T> Ratelimit<T> {
        /// Create a new ratelimit with at most `count` uses in `wait_time`.
        pub fn new(inner: T, count: usize, wait_time: Duration) -> Self {
            let (send, recv) = channel(count);
            (0..count).for_each(|_| {
                send.send(()).ok();
            });
            Self {
                inner,
                send,
                recv,
                wait_time,
            }
        }

        /// Borrow the inner `T`. You can only hol this reference `count` times in `wait_time`.
        /// The clock counts from the moment the ref is dropped.
        pub async fn borrow<'a>(&'a self) -> impl Deref<Target = T> + 'a {
            self.recv.recv_async().await.unwrap();
            RatelimitGuard {
                inner: &self.inner,
                send: &self.send,
                wait_time: &self.wait_time,
            }
        }
    }

    impl<'a, T> Deref for RatelimitGuard<'a, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            self.inner
        }
    }

    impl<'a, T> Drop for RatelimitGuard<'a, T> {
        fn drop(&mut self) {
            let send = self.send.clone();
            let wait_time = self.wait_time.clone();
            tokio::spawn(async move {
                tokio::time::sleep(wait_time).await;
                send.send_async(()).await.ok();
            });
        }
    }
}
