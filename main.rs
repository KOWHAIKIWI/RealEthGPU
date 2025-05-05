mod gpu_wrapper;

use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::thread::sleep;
use serde::{Serialize, Deserialize};
use std::env;
use std::path::Path;

const KNOWN_WORDS: [&str; 8] = [
    
];

// Leave empty for zero mode, insert partial/full hex address otherwise
const TARGET_ADDRESS: &str = "93262cf869dc18b84ca6b600a4155eeebdf42882";

#[derive(Serialize, Deserialize)]
struct Progress {
    seeds_tried: u64,
    valid_checksums: u64,
}

fn main() {
    println!("🚀 Launching seeds recovery...");

    let args: Vec<String> = env::args().collect();
    let worker_id: u32 = if args.len() > 1 {
        args[1].parse().expect("Worker ID must be a number")
    } else {
        0
    };

    let mut match_mode = 0;
    let mut match_prefix_len = 20;

    let target_trimmed = TARGET_ADDRESS.trim();

    if target_trimmed.is_empty() {
        match_mode = 2;
        match_prefix_len = 0;
        println!("🟡 No address provided. Running in ZERO address mode.");
    } else if target_trimmed.len() < 40 {
        match_mode = 1;
        match_prefix_len = (target_trimmed.len() / 2) as i32;
        println!("🟠 Partial address detected. Matching first {} bytes.", match_prefix_len);
    } else {
        match_mode = 0;
        match_prefix_len = 20;
        println!("🟢 Full address mode activated.");
    }

    let start_time = Instant::now();
    let mut last_heartbeat = Instant::now();
    let mut seeds_tried: u64 = 0;
    let mut valid_checksums: u64 = 0;

    let words = load_words("words.txt");

    gpu_wrapper::init_gpu(&words, &KNOWN_WORDS, TARGET_ADDRESS);

    if let Ok(file) = File::open(format!("progress_{}.json", worker_id)) {
        if let Ok(prog) = serde_json::from_reader::<_, Progress>(BufReader::new(file)) {
            seeds_tried = prog.seeds_tried;
            valid_checksums = prog.valid_checksums;
        }
    }

    let mut last_save = Instant::now();

    loop {
        let total_workers = 12; // or load from env/config if you want
        let (batch_seeds, batch_valid) = gpu_wrapper::launch_batch(worker_id, total_workers, match_mode, match_prefix_len);

        seeds_tried += batch_seeds;
        valid_checksums += batch_valid;

        if batch_valid > 0 {
            println!("🚨 Seed found by this worker! Creating found.flag...");
            let _ = File::create("found.flag");
        }

        if Path::new("found.flag").exists() {
            println!("🚨 Detected found.flag. Another miner succeeded. Shutting down...");
            break;
        }

        if last_heartbeat.elapsed().as_secs() >= 5 {
            let elapsed_secs = start_time.elapsed().as_secs();
            let seeds_per_sec = if elapsed_secs > 0 { seeds_tried / elapsed_secs } else { 0 };
            let est_total_candidates = 2048u64 * 2048u64;
            let remaining = est_total_candidates.saturating_sub(seeds_tried);
            let eta_secs = if seeds_per_sec > 0 { remaining / seeds_per_sec } else { 0 };

            println!(
                "[Worker {}] [Batch {}] _ Seeds tried: {} | __ Valid checksums: {} | __ Speed: {} seeds/sec | __ ETA: ~{}h {}m",
                worker_id,
                seeds_tried / 50_000_000,
                seeds_tried,
                valid_checksums,
                seeds_per_sec,
                eta_secs / 3600,
                (eta_secs % 3600) / 60
            );

            last_heartbeat = Instant::now();
        }

        if last_save.elapsed().as_secs() > 60 {
            let prog = Progress {
                seeds_tried,
                valid_checksums,
            };
            if let Ok(mut file) = File::create(format!("progress_{}.json", worker_id)) {
                let _ = file.write_all(serde_json::to_string(&prog).unwrap().as_bytes());
            }
            last_save = Instant::now();
        }

        sleep(Duration::from_millis(100));
    }
}

fn load_words(path: &str) -> Vec<String> {
    let file = File::open(path).expect("words.txt missing!");
    BufReader::new(file).lines().map(|l| l.unwrap()).collect()
}
