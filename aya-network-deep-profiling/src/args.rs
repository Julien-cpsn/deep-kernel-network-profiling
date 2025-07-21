use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub trace: bool,

    #[command(flatten)]
    pub verbosity: Verbosity,
}
