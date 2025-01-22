// use chirp_workflow::prelude::*;

// #[derive(Debug, Default)]
// pub struct Input {}

// #[derive(Debug)]
// pub struct Output {
// 	pub user_id: Uuid,
// }

// #[operation]
// async fn user(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output>  {
// 	let user_id = Uuid::new_v4();

// 	// TODO: Move back to faker op after ops can dispatch workflows
//     let mut creation_sub = ctx
//         .subscribe::<crate::workflows::user::CreateComplete>(("user_id", user_id))
//         .await?;

//     ctx.workflow(crate::workflows::user::Input {
// 		user_id,
// 		display_name: None,
// 	})
// 	.tag("user_id", user_id)
// 	.dispatch()
// 	.await?;

//     creation_sub.next().await?;

// 	Ok(Output {
// 		user_id: user_id,
// 	})
// }
