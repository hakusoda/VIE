use clap::{ Parser, Subcommand };

mod data;
mod http;

#[derive(Debug, Parser)]
struct Cli {
	#[command(subcommand)]
	command: Command
}

#[derive(Debug, Subcommand)]
enum Command {
	#[command(about = "Begin sync for single instance tree")]
	Single
}

#[tokio::main]
async fn main() {
	let arguments = Cli::parse();
	match arguments.command {
		Command::Single => {
			http::start_server().await.unwrap();
		}
	}
}