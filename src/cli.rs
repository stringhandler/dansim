use clap_derive::parser as Parser;

#[derive(Parser, Debug)]
pub struct Cli {

    #[clap(short, long, default_value = "4")]
    num_vns: usize,
    #[clap(long, default_value = "100ms")]
    min_latency: humantime::Duration,
    #[clap(long, default_value = "100ms")]
    max_latency: humantime::Duration,
    #[clap(long, default_value = "10")]
    max_block_size: usize,
}
