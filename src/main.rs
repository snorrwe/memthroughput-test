use std::io::Write as _;

use clap::Parser;
use clap_derive::{Parser, Subcommand};

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

fn fmt_bytes(mut w: impl std::io::Write, bytes: f64) -> std::io::Result<()> {
    if bytes < 1000.0 {
        write!(w, "{bytes:.2} bytes")
    } else if bytes < 1000e3 {
        write!(w, "{:.2} KB", bytes / 1e3)
    } else if bytes < 1000e6 {
        write!(w, "{:.2} MB", bytes / 1e6)
    } else if bytes < 1000e9 {
        write!(w, "{:.2} GB", bytes / 1e9)
    } else if bytes < 1000e12 {
        write!(w, "{:.2} TB", bytes / 1e12)
    } else {
        write!(w, "{bytes:.2} bytes")
    }
}

fn print_throughput_ghz(bytes_per_sec: f64) {
    let mut stdout = std::io::stdout().lock();

    fmt_bytes(&mut stdout, bytes_per_sec).unwrap();
    writeln!(&mut stdout, "/s").unwrap();
}

fn memcpy_test(size: usize, threads: usize, repetitions: usize, warmups: usize) {
    let mut src = Vec::<u8>::with_capacity(size);
    let mut dst = Vec::<u8>::with_capacity(size);

    // it's important to touch all allocated pages, we don't want to count the page faults the
    // first time they're used
    // also, if we have initialized vectors, then we can use the nice slice APIs
    src.resize(size, 0xBE);
    dst.resize(size, 0xEF);

    println!("memcpy test of {size} bytes on {threads} thread(s)");
    for i in 0..repetitions {
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

    println!("memset test of {size} bytes on {threads} thread(s)");
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
                for b in buf.chunks_mut(chunk_size) {
                    s.spawn(|| {
                        latch.count_down();
                        latch.wait();
                        b.fill(0xFE);
                    });
                }
                latch.count_down();
                latch.wait();
                start = std::time::Instant::now();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_formatting() {
        let mut result = Vec::new();
        fmt_bytes(&mut result, 13980987619.602848).unwrap();

        let result = String::from_utf8(result).unwrap();

        assert_eq!(result, "13.98 GB");
    }
}
