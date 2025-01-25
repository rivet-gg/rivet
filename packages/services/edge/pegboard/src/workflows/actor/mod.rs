use chirp_workflow::prelude::*;

mod migrations;

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
	pub actor_id: Uuid,
}

#[workflow]
pub async fn pegboard_actor(ctx: &mut WorkflowCtx, input: &Input) -> GlobalResult<()> {
	migrations::run(ctx).await?;

	Ok(())
}
