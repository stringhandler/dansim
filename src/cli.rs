use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(short, long, default_value = "8")]
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
    #[clap(long, default_value = "2000ms")]
    pub delta: humantime::Duration,

    #[clap(long, default_value = "40")]
    pub num_steps: usize,
    #[clap(long, default_value = "100ms")]
    pub time_per_step: humantime::Duration,

    #[clap(long, default_value = "1")]
    pub print_stats_every: usize,

    #[clap(long, default_value = "1")]
    pub max_tx_per_step_per_block: usize,
}
