//! A crate for Random X mining.

#![deny(missing_docs)]

mod job;
mod share;
mod stratum;
mod worker;

use crate::{stratum::Stratum, worker::Worker};
use std::{
    io,
    num::NonZeroUsize,
    time::{Duration, Instant},
};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(60);

/// The miner contains all the mining info.
pub struct Miner {
    /// The mining pool URL including the port.
    pub url: String,
    /// The wallet address.
    pub user: String,
    /// The worker name.
    pub pass: String,
    /// The number of CPU threads.
    pub threads: NonZeroUsize,
    /// Switch to light mode.
    pub light: bool,
}

impl Miner {
    /// Runs the miner using the specified settnigs.
    pub fn run(&self) -> io::Result<()> {
        tracing_subscriber::fmt()
            .pretty()
            .with_max_level(tracing::Level::DEBUG)
            .with_file(false)
            .with_line_number(false)
            .init();

        let Self {
            url,
            user,
            pass,
            light,
            threads,
        } = self;

        let mut stratum = Stratum::login(url, user, pass)?;
        let worker = Worker::init(stratum.try_recv_job().unwrap(), *threads, !light);
        let mut timer = Instant::now();

        loop {
            if let Ok(job) = stratum.try_recv_job() {
                worker.work(job);
            }
            if let Ok(share) = worker.try_recv_share() {
                stratum.submit(share)?;
            }
            if timer.elapsed() >= KEEP_ALIVE_INTERVAL {
                timer = Instant::now();
                stratum.keep_alive()?;
            }
        }
    }
}
