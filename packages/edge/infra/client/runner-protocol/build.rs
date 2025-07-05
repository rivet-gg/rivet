use std::io::Result;

fn main() -> Result<()> {
	prost_build::compile_protos(
		&[
			"resources/proto/kv.proto",
			"resources/proto/runner_protocol.proto",
		],
		&["resources/proto/"],
	)?;

	Ok(())
}
