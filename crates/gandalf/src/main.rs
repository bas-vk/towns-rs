mod args;
mod config;
mod stream;

use clap::Parser;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = args::Opts::parse();
    let cfg = config::config(opts.network);

    match opts.command {
        args::Commands::Stream(args) => args.execute(&cfg).await,
        args::Commands::Miniblock(args) => args.execute(&cfg).await,
        args::Commands::Node(args) => args.execute(&cfg).await,
    }
}
