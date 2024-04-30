use std::collections::HashMap;

use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
// use chirp_worker::prelude::*;

#[operation(name = "servers-server-create")]
pub async fn handle(
	ctx: OperationContext<servers::server_create::Request>,
) -> GlobalResult<servers::server_create::Response> {
	todo!();
}
