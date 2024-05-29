use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker_test]
async fn basic(ctx: TestCtx) {
	// TODO: Create opengb env, send request to it, read its tail event, delete env
}
