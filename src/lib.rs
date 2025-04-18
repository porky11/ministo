extern crate core;

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

pub struct Miner {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub threads: NonZeroUsize,
    pub light: bool,
}

impl Miner {
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
