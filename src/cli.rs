use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(short, long, default_value = "4")]
    pub num_vns: usize,
    #[clap(long, default_value = "50ms")]
    pub min_latency: humantime::Duration,
    #[clap(long, default_value = "50ms")]
    pub max_latency: humantime::Duration,
    #[clap(long, default_value = "10")]
    pub max_block_size: usize,
    #[clap(long, default_value = "1")]
    pub num_shards: u32,

    ///  The time before deciding a block has timed out
    #[clap(long, default_value = "5000ms")]
    pub delta: humantime::Duration,

    #[clap(long, default_value = "6")]
    pub num_steps: usize,
    #[clap(long, default_value = "100ms")]
    pub time_per_step: humantime::Duration,

    #[clap(long, default_value = "10")]
    pub print_stats_every: usize,

    #[clap(long, default_value = "5")]
    pub max_tx_per_step_per_block: usize,

    #[clap(long, default_value = "50")]
    pub num_transactions: usize,

    #[clap(long, default_value = "5")]
    pub probability_2_shards: u32,
    #[clap(long, default_value = "0")]
    pub probability_3_shards: u32,
    #[clap(long, default_value = "0")]
    pub probability_4_shards: u32,
    #[clap(long, default_value = "0")]
    pub probability_5_shards: u32,
}
