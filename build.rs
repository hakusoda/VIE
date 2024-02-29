use std::process::Command;
fn main() {
    let git_hash = Command::new("git")
		.args(&["rev-parse", "--short", "HEAD"])
		.output()
		.unwrap()
		.stdout;
    println!("cargo:rustc-env=GIT_HASH={}", String::from_utf8(git_hash).unwrap());
	println!("cargo:rerun-if-changed=.git/HEAD");
}