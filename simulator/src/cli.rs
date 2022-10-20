use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, help = "Path to task json file")]
    pub task: Option<String>,

    #[arg(short, long, help = "Path to solution json file")]
    pub solution: Option<String>,

    #[arg(
        short,
        long,
        help = "Path to combined task/solution json file (from 'cli' export)"
    )]
    pub cli: Option<String>,
}
