use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio::task;
use std::fmt;

#[derive(Debug)]
pub struct ErrNoUpdate;

impl fmt::Display for ErrNoUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no update available")
    }
}

impl Error for ErrNoUpdate {}

pub trait Logger {
    fn debugw(&self, message: &str);
    fn errorw(&self, message: &str, error: &dyn Error);
}

pub trait Source {
    fn fetch(&self, ctx: &tokio::time::Instant) -> Result<Box<dyn std::any::Any>, Box<dyn Error>>;
}

pub trait Updater {}

pub struct SourcePoller {
    log: Arc<dyn Logger>,
    source: Arc<dyn Source>,
    updates: Sender<Box<dyn std::any::Any>>,
    poll_interval: Duration,
    fetch_timeout: Duration,
}

impl SourcePoller {
    pub fn new(
        source: Arc<dyn Source>,
        log: Arc<dyn Logger>,
        poll_interval: Duration,
        fetch_timeout: Duration,
        buffer_capacity: usize,
    ) -> Self {
        let (tx, _rx) = mpsc::channel();
        Self {
            log,
            source,
            updates: tx,
            poll_interval,
            fetch_timeout,
        }
    }

    pub fn run(self: Arc<Self>) {
        let log = self.log.clone();
        let source = self.source.clone();
        let updates = self.updates.clone();
        let poll_interval = self.poll_interval;
        let fetch_timeout = self.fetch_timeout;

        thread::spawn(move || {
            log.debugw("poller started");
            let initial_fetch = Self::execute_fetch(&source, fetch_timeout);
            match initial_fetch {
                Ok(data) => {
                    let _ = updates.send(data);
                }
                Err(err) => {
                    if err.is::<ErrNoUpdate>() {
                        log.debugw("no update found on initial fetch");
                    } else {
                        log.errorw("failed initial fetch", &*err);
                    }
                }
            }

            let mut reused_timer = Instant::now();
            loop {
                if reused_timer.elapsed() >= poll_interval {
                    let fetch_result = Self::execute_fetch(&source, fetch_timeout);
                    match fetch_result {
                        Ok(data) => {
                            let _ = updates.send(data);
                        }
                        Err(err) => {
                            if err.is::<ErrNoUpdate>() {
                                log.debugw("no update found");
                            } else {
                                log.errorw("failed to fetch from source", &*err);
                            }
                        }
                    }
                    reused_timer = Instant::now();
                }
                thread::sleep(Duration::from_millis(100)); // Prevent busy waiting
            }
        });
    }

    fn execute_fetch(source: &Arc<dyn Source>, fetch_timeout: Duration) -> Result<Box<dyn std::any::Any>, Box<dyn Error>> {
        let start = Instant::now();
        let ctx = start;

        let result = timeout(fetch_timeout, async {
            source.fetch(&ctx).await
        });

        match result {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(Box::new(ErrNoUpdate)),
        }
    }

    pub fn updates(&self) -> Receiver<Box<dyn std::any::Any>> {
        self.updates.clone()
    }
}

