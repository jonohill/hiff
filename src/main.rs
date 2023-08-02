use std::thread::sleep;
use std::time::{Instant, Duration};

use clap::Parser;
use clap::ArgAction::Count;

const DOMAINS_RAW: &str = include_str!("domains.txt");

#[derive(Parser)]
struct Args {

    /// Zero or more domains to check.
    /// If not set, a list of top domains will be used.
    #[clap(name = "DOMAIN")]
    domains: Vec<String>,

    /// Stop after this many tests have been run.
    #[clap(short, long)]
    count: Option<u64>,

    /// Wait this many milliseconds between tests.
    #[clap(short = 'i', long, default_value = "1000")]
    wait: u64,

    /// Wait this many milliseconds for each test to complete.
    #[clap(short, long, default_value = "1000")]
    timeout: u64,

    /// Be more verbose.
    #[clap(short, long, action = Count, default_value = "0")]
    verbose: u8,
}

fn main() {
    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(match args.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    })).init();
    
    let domains = if args.domains.is_empty() {
        DOMAINS_RAW.lines().map(|s| s.to_string()).collect()
    } else {
        args.domains
    };
    let wait = Duration::from_millis(args.wait);
    let count = args.count.unwrap_or(u64::MAX);

    let mut bad_domains = Vec::<String>::new();

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(args.timeout))
        .build()
        .unwrap();
    

    let mut seq = 0;
    for round in 0.. {
        for domain in &domains {
            if bad_domains.contains(domain) {
                continue;
            }

            let start_time = Instant::now();
            let result = client.head(&format!("http://{}", domain)).send();
            let elapsed = start_time.elapsed().as_millis();

            match result {
                Ok(response) => {
                    // pretty rough
                    let size: u64 = response.headers().iter()
                        .map(|(k, v)| format!("{}:{}", k, v.to_str().unwrap_or("")).len() as u64)
                        .sum();
                    println!("{} bytes from {}: seq={} time={}ms", size, domain, seq, elapsed);
                }
                Err(err) => {

                    if round == 0 {
                        log::info!("{} failed its very first request and will be ignored", domain);
                        bad_domains.push(domain.clone());
                    } else if err.is_timeout() {
                        println!("Request timeout for {} seq {}", domain, seq);
                    } else if err.is_connect() {
                        println!("Connection error for {} seq {}", domain, seq);
                    } else {
                        println!("Unknown error for {} seq {}", domain, seq);
                    }

                }
            }

            sleep(wait);

            seq += 1;
            if seq >= count {
                break;
            }
        }
    }


}
