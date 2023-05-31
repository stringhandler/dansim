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
    #[clap(long, default_value = "2")]
    pub num_shards: u32,

    ///  The time before deciding a block has timed out
    #[clap(long, default_value = "1s")]
    pub delta: humantime::Duration,
}
