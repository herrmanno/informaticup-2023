use clap::clap_derive::ValueEnum;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(long, help = "Runtime in seconds")]
    pub time: Option<u64>,

    #[arg(long, help = "Number of cores to use")]
    pub cores: Option<usize>,

    #[arg(long, help = "Output format")]
    pub out: Option<OutputFormat>,

    #[arg(long, help = "Seed for rng")]
    pub seed: Option<u64>,

    #[arg(long, help = "Print additional solution stats")]
    pub stats: bool,

    #[arg(long, help = "Print final result as map")]
    pub print: bool,
}

impl Args {
    pub fn output_format(&self) -> OutputFormat {
        self.out.clone().unwrap_or(OutputFormat::Solution)
    }
}

#[derive(ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Cli,
    Solution,
}
