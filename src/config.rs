use std::{
	fs::File,
	io::BufReader,
	path::{ Path, PathBuf }
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
	#[serde(default)]
	pub root_path: Option<PathBuf>,

	#[serde(default)]
	pub compatibility: CompatibilityConfig
}

impl Default for Config {
	fn default() -> Self {
		Self {
			root_path: None,
			compatibility: CompatibilityConfig::default()
		}
	}
}

#[derive(Deserialize)]
pub struct CompatibilityConfig {
	#[serde(default)]
	pub rojo_sourcemap: bool
}

impl Default for CompatibilityConfig {
	fn default() -> Self {
		Self {
			rojo_sourcemap: false
		}
	}
}

pub fn read_config_file<P: AsRef<Path>>(file_path: P) -> Config {
	if let Ok(file) = File::open(file_path) {
		serde_yaml::from_reader(BufReader::new(file)).ok().unwrap_or_else(|| Config::default())
	} else { Config::default() }
}