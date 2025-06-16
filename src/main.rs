mod fmtbytes;

use clap::Parser;
use clap_derive::{Parser, Subcommand};

use crate::fmtbytes::Bytes;

#[derive(Debug, Subcommand)]
enum Cmd {
    Memcpy {
        /// buffer size in bytes
        #[arg(short, long, default_value_t = 1048576)]
        size: usize,
        /// number of threads to perform the copy on, splitting the buffer into threads number of
        /// chunks
        #[arg(short, long, default_value_t = std::thread::available_parallelism().map(|x|x.get()).unwrap_or(1))]
        threads: usize,
    },
    Memset {
        /// buffer size in bytes
        #[arg(short, long, default_value_t = 1048576)]
        size: usize,
        /// number of threads to perform the memset on, splitting the buffer into threads number of
        /// chunks
        #[arg(short, long, default_value_t = std::thread::available_parallelism().map(|x|x.get()).unwrap_or(1))]
        threads: usize,
    },
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
    #[arg(short, long, default_value_t = 5)]
    repetitions: usize,

    /// number of iterations which are not reported
    #[arg(short, long, default_value_t = 5)]
    warmups: usize,
}

fn print_throughput_ghz(bytes_per_sec: f64) {
    println!("{}/s", Bytes(bytes_per_sec));
}

fn memcpy_test(size: usize, threads: usize, repetitions: usize, warmups: usize) {
    let mut src = Vec::<u8>::with_capacity(size);
    let mut dst = Vec::<u8>::with_capacity(size);

    // it's important to touch all allocated pages, we don't want to count the page faults the
    // first time they're used
    // also, if we have initialized vectors, then we can use the nice slice APIs
    src.resize(size, 0xBE);
    dst.resize(size, 0xEF);

    println!(
        "memcpy test of {} on {threads} thread(s)",
        Bytes(size as f64)
    );
    for i in 0..repetitions {
        let mut start = std::time::Instant::now();
        let end;
        if threads <= 1 {
            dst.copy_from_slice(src.as_slice());
            end = std::time::Instant::now();
        } else {
            let num_threads = threads;

            let latch = latches::sync::Latch::new(num_threads + 1);

            end = std::thread::scope(|s| {
                let chunk_size = size.div_ceil(num_threads);
                debug_assert!(chunk_size * num_threads >= size);
                let mut threads = Vec::with_capacity(num_threads);
                for (src, dst) in src.chunks(chunk_size).zip(dst.chunks_mut(chunk_size)) {
                    threads.push(s.spawn(|| {
                        latch.count_down();
                        latch.wait();
                        let start = std::time::Instant::now();
                        dst.copy_from_slice(src);
                        (start, std::time::Instant::now())
                    }));
                }
                latch.count_down();
                latch.wait();
                start = std::time::Instant::now();

                let mut end = start;
                for t in threads {
                    let (tstart, tend) = t.join().unwrap();
                    start = start.min(tstart);
                    end = end.max(tend);
                }
                end
            });
        }

        if i >= warmups {
            let dur = end - start;
            print!("throughput: ");
            print_throughput_ghz(size as f64 / dur.as_secs_f64());
        }
    }
}

fn memset_test(size: usize, threads: usize, repetitions: usize, warmups: usize) {
    let mut buf = Vec::<u8>::with_capacity(size);

    // it's important to touch all allocated pages, we don't want to count the page faults the
    // first time they're used
    // also, if we have initialized vectors, then we can use the nice slice APIs
    buf.resize(size, 0xBE);

    println!(
        "memset test of {} on {threads} thread(s)",
        Bytes(size as f64)
    );
    for i in 0..repetitions {
        let mut start = std::time::Instant::now();
        let end;
        if threads <= 1 {
            buf.as_mut_slice().fill(0xFE);
            end = std::time::Instant::now();
        } else {
            let num_threads = threads;

            let latch = latches::sync::Latch::new(num_threads + 1);

            std::thread::scope(|s| {
                let chunk_size = size.div_ceil(num_threads);
                debug_assert!(chunk_size * num_threads >= size);
                let mut threads = Vec::with_capacity(num_threads);
                for b in buf.chunks_mut(chunk_size) {
                    threads.push(s.spawn(|| {
                        latch.count_down();
                        latch.wait();
                        let start = std::time::Instant::now();
                        b.fill(0xFE);
                        (start, std::time::Instant::now())
                    }));
                }
                latch.count_down();
                latch.wait();
                start = std::time::Instant::now();
                let mut end = start;
                for t in threads {
                    let (tstart, tend) = t.join().unwrap();
                    start = start.max(tstart);
                    end = end.max(tend);
                }
                end
            });
            end = std::time::Instant::now();
        }

        if i >= warmups {
            let dur = end - start;
            print!("throughput: ",);
            print_throughput_ghz(size as f64 / dur.as_secs_f64());
        }
    }
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Cmd::Memcpy { size, threads } => {
            memcpy_test(size, threads, args.repetitions + args.warmups, args.warmups)
        }
        Cmd::Memset { size, threads } => {
            memset_test(size, threads, args.repetitions + args.warmups, args.warmups)
        }
    }
}
