use clap::{Parser, Subcommand};
use executer::Executer;
use futures::{pin_mut, StreamExt};
use reqwest::Url;

mod executer;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run tests
    Run,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Run => {
            let executer = Executer::new(reqwest::Client::new());
            let steps = vec![
                executer::Step::new(Url::parse("http://localhost:8080/hoge").unwrap(), "GET"),
                executer::Step::new(Url::parse("http://localhost:8080/hoge").unwrap(), "GET"),
            ];
            let results = executer.execute_steps(steps);
            pin_mut!(results);
            while let Some(result) = results.next().await {
                println!("{:?}", result);
            }
        }
    }
}
