use tokio::{fs, process::Command};

#[tokio::main]
async fn main() {
	let current_dir = std::env::current_dir().unwrap();
	let server_install_path = {
		let mut dir = current_dir.clone();
		dir.pop();

		dir.join("worker")
			.join("src")
			.join("workers")
			.join("server_install")
	};

	// Add rereun statement
	println!("cargo:rerun-if-changed={}", server_install_path.display(),);

	// Get a hash of the server install worker
	let git_hash_cmd = Command::new("git")
		.arg("rev-parse")
		.arg("HEAD:svc/pkg/cluster/worker/src/workers/server_install")
		.output()
		.await
		.unwrap();

	if !git_hash_cmd.status.success() {
		panic!(
			"failed to get git hash ({}):\n{}",
			git_hash_cmd.status,
			String::from_utf8(git_hash_cmd.stderr).unwrap()
		);
	}

	let hash = String::from_utf8(git_hash_cmd.stdout).unwrap();

	fs::create_dir_all(current_dir.join("gen")).await.unwrap();
	fs::write(current_dir.join("gen").join("hash.txt"), hash.trim())
		.await
		.unwrap();
}
