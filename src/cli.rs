use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {

    #[clap(short, long, default_value = "4")]
    pub num_vns: usize,
    #[clap(long, default_value = "100ms")]
    pub min_latency: humantime::Duration,
    #[clap(long, default_value = "100ms")]
    pub max_latency: humantime::Duration,
    #[clap(long, default_value = "10")]
    pub max_block_size: usize,
}
