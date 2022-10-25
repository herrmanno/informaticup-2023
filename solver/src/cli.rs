use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(long, help = "Path to task json file")]
    pub task: Option<String>,

    #[arg(long, help = "Path to solution json file")]
    pub solution: Option<String>,

    #[arg(
        long,
        help = "Path to combined task/solution json file (from 'cli' export)"
    )]
    pub cli: Option<String>,

    #[arg(long, help = "Runtime limit")]
    pub time: Option<u64>,

    #[arg(long, help = "Number of cores to use")]
    pub cores: Option<usize>,

    #[arg(long, help = "Print output in CLI format instead of solution format")]
    pub cli_out: bool,

    #[arg(long, help = "Print additional solution stats")]
    pub stats: bool,

    #[arg(long, help = "Print final result as map")]
    pub print: bool,
}
