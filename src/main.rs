use clap::Parser;
use clap_derive::{Parser, Subcommand};
use xshell::Shell;

#[derive(Debug, Subcommand)]
enum Cmd {
    Memcpy {
        /// buffer size in bytes
        #[arg(short, long, default_value_t = 1048576)]
        size: usize,
        /// number of threads to perform the copy on
        #[arg(short, long, default_value_t = std::thread::available_parallelism().map(|x|x.get()).unwrap_or(1))]
        threads: usize,
    },
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

fn print_throughput_ghz(bytes_per_sec: f64) {
    let sh = Shell::new().unwrap();

    let hz = bytes_per_sec.to_string();
    if let Err(err) = xshell::cmd!(sh, "units -o'%.2f' {hz}bytes/s GB/s").run() {
        eprintln!("Failed to convert bytes per sec to GB per sec: {err:?}");
    }
}

fn memcpy_test(size: usize, threads: usize) {
    let mut src = Vec::<u8>::with_capacity(size);
    let mut dst = Vec::<u8>::with_capacity(size);

    // it's important to touch all allocated pages, we don't want to count the page faults the
    // first time they're used
    // also, if we have initialized vectors, then we can use the nice slice APIs
    src.resize(size, 0xBE);
    dst.resize(size, 0xEF);

    let mut start = std::time::Instant::now();
    let end;
    if threads <= 1 {
        dst.copy_from_slice(src.as_slice());
        end = std::time::Instant::now();
    } else {
        let num_threads = threads;

        let latch = latches::sync::Latch::new(num_threads + 1);

        std::thread::scope(|s| {
            let chunk_size = size.div_ceil(num_threads);
            debug_assert!(chunk_size * num_threads >= size);
            for (src, dst) in src.chunks(chunk_size).zip(dst.chunks_mut(chunk_size)) {
                s.spawn(|| {
                    latch.count_down();
                    latch.wait();
                    dst.copy_from_slice(src);
                });
            }
            latch.count_down();
            latch.wait();
            start = std::time::Instant::now();
        });
        end = std::time::Instant::now();
    }

    let dur = end - start;
    println!("memcpy test of {size} bytes on {threads} threads duration: {dur:?}");
    println!(
        "throughput: {} bytes per second",
        size as f64 / dur.as_secs_f64()
    );
    print_throughput_ghz(size as f64 / dur.as_secs_f64());
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Cmd::Memcpy { size, threads } => memcpy_test(size, threads),
    }
}
