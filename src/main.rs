use clap::{ Parser, Subcommand };

mod data;
mod http;
mod config;
mod export;
mod import;
mod compatibility;

#[derive(Debug, Parser)]
struct Cli {
	#[command(subcommand)]
	command: Command
}

#[derive(Debug, Subcommand)]
enum Command {
	#[command(about = "Begin sync for single instance tree")]
	Single,
	
	#[command(about = "Print version information")]
	Version
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_HASH: &str = env!("GIT_HASH");

#[tokio::main]
async fn main() {
	let arguments = Cli::parse();
	match arguments.command {
		Command::Single => {
			http::start_server().await.unwrap();
		},
		Command::Version => println!("VIE {VERSION} ({GIT_HASH})")
	}
}

#[macro_export]
macro_rules! cast {
	($target: expr, $pat: path) => {
		if let $pat(a) = $target {
			a
		} else {
			panic!("mismatch variant when cast to {}", stringify!($pat));
		}
	};
}