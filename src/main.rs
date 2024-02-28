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
	#[command(about = "Start server for the Roblox Studio plugin (only one action)")]
	Once
}

#[tokio::main]
async fn main() {
	let arguments = Cli::parse();
	match arguments.command {
		Command::Once => {
			http::start_server().await.unwrap();
		}
	}
}