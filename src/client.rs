use reqwest::Client as HTTP;
/// Client represents a Codeforces API client.
/// It wraps around a reqwest HTTP client and provides rate-limiting.
pub struct Client(rate_limit::RateLimit<HTTP>);

// Number of requests per second to be rate-limited.
pub const REQUESTS_PER_SECOND: u64 = 4;

impl Client {
    /// New creates a new Client.
    pub fn new(http: HTTP) -> Self {
        Self(rate_limit::RateLimit::new(
            http,
            REQUESTS_PER_SECOND,
            std::time::Duration::from_secs(1),
        ))
    }

    /// Borrows and returns the inner HTTP client.
    pub async fn borrow(&self) -> &HTTP {
        self.0.borrow().await
    }

    /// Runs reqwest::Client::execute.
    pub async fn execute(&self, req: reqwest::Request) -> reqwest::Result<reqwest::Response> {
        self.borrow().await.execute(req).await
    }

    /// Runs reqwest::Client::request.
    pub async fn request(
        &self,
        method: reqwest::Method,
        url: impl reqwest::IntoUrl,
    ) -> reqwest::RequestBuilder {
        self.borrow().await.request(method, url)
    }
}

mod rate_limit {
    use futures_util::lock::Mutex;
    use std::time::{Duration, Instant};
    pub struct RateLimit<T> {
        item: T,
        clock: Mutex<(u64, Instant)>,
        lock_count: u64,
        lock_duration: Duration,
    }

    impl<T: 'static> RateLimit<T> {
        pub fn new(item: T, count: u64, duration: Duration) -> Self
        where
            T: 'static,
        {
            Self {
                item,
                clock: Mutex::new((count, Instant::now() + duration)),
                lock_count: count,
                lock_duration: duration,
            }
        }

        pub async fn borrow(&self) -> &T {
            self.wait_for_clock().await;
            &self.item
        }

        // pub async fn borrow_mut(&mut self) -> &mut T {
        //     self.wait_for_clock().await;
        //     &mut self.item
        // }

        async fn wait_for_clock(&self) {
            let mut clock = self.clock.lock().await;
            let (times_left, next_clock) = *clock;
            if times_left == 0 {
                futures_timer::Delay::new(Instant::now() - next_clock).await;
                *clock = (self.lock_count, Instant::now() + self.lock_duration);
            } else {
                *clock = (times_left - 1, next_clock);
            }
        }
    }
}
