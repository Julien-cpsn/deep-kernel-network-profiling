use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub trace: bool,

    #[arg(long, default_value_t = 5_000_000_000)]
    pub max_time: u64,

    #[command(flatten)]
    pub verbosity: Verbosity,
}
