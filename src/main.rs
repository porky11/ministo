mod job;
mod share;
mod stratum;
mod worker;

use crate::stratum::Stratum;
use crate::worker::{Mode, Worker};
use clap::Parser;
use std::io;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(60);
#[derive(Parser)]
struct Args {
    /// Pool address (URL:PORT)
    #[arg(short = 'o', long, default_value = "gulf.moneroocean.stream:10001")]
    url: String,
    /// Wallet address
    #[arg(
        short,
        long,
        default_value = "8571HAJKFudM4Y12Q9WLaMGgsiMyJCkvZaw38iy1ufMAXRuZbdfVekH7Ab6UdDFVfJbrASauW4iU69nM6dZ2A4hv1dBnRsp"
    )]
    user: String,
    /// Worker name
    #[arg(short, long, default_value = "x")]
    pass: String,
    /// Number of CPU threads
    #[arg(short, long, default_value_t = all_threads())]
    threads: NonZeroUsize,
    /// Switch to light mode
    #[arg(long)]
    light: bool,
}

fn all_threads() -> NonZeroUsize {
    std::thread::available_parallelism().unwrap()
}

fn main() -> io::Result<()> {
    let Args {
        url,
        user,
        pass,
        light,
        threads,
    } = Args::parse();

    let mode = if light { Mode::Light } else { Mode::Fast };
    let mut stratum = Stratum::login(&url, &user, &pass)?;
    let mut worker = Worker::init(stratum.try_recv_job().unwrap(), mode, threads);
    let mut timer = Instant::now();

    loop {
        if let Ok(job) = stratum.try_recv_job() {
            println!("New Job!");
            worker.work(job);
        }
        if let Ok(share) = worker.try_recv_share() {
            println!("Submit share request sending...");
            stratum.submit(share)?;
        }
        if timer.elapsed() > KEEP_ALIVE_INTERVAL {
            println!("Keep Alive request sending...");
            timer = Instant::now();
            stratum.keep_alive()?;
        }
    }
}
