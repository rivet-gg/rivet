use futures_util::FutureExt;
use std::collections::HashMap;

use chirp_workflow::prelude::*;

#[derive(Debug)]
pub struct Input {
	pub build_id: Uuid,
	pub tags: HashMap<String, Option<String>>,
	pub exclusive_tags: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Output {}

#[operation]
pub async fn patch_tags(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	let exclusive_tags_map = if let Some(exclusive_tags) = &input.exclusive_tags {
		// Validate tags don't overlap
		let exclusive_tags_map = exclusive_tags
			.iter()
			.map(|tag| {
				let value = unwrap_with!(
					unwrap_with!(input.tags.get(tag), BUILD_TAGS_MISSING_EXCLUSIVE_KEY),
					BUILD_TAGS_NULL_EXCLUSIVE_KEY
				);

				Ok((tag.clone(), value.clone()))
			})
			.collect::<GlobalResult<HashMap<_, _>>>()?;

		Some(exclusive_tags_map)
	} else {
		None
	};

	let tags_json = util::serde::Raw::new(&input.tags)?;

	rivet_pools::utils::crdb::tx(&ctx.crdb().await?, |tx| {
		let ctx = ctx.clone();
		let build_id = input.build_id;
		let tags_json = tags_json.clone();
		let exclusive_tags_map = exclusive_tags_map.clone();

		async move {
			// Remove the exclusive tag from other builds of the same owner (same game id OR same env id)
			if let Some(exclusive_tags_map) = exclusive_tags_map {
				// Fetch all builds for this env or game
				let exclusive_tags_json = util::serde::Raw::new(&exclusive_tags_map)?;
				let all_builds = sql_fetch_all!(
					[ctx, (Uuid, serde_json::Value), @tx tx]
					"
					WITH
						build_data AS (
							SELECT game_id, env_id
							FROM db_build.builds
							WHERE build_id = $1
						)
					SELECT b.build_id, b.tags
					FROM db_build.builds AS b, build_data
					WHERE
						-- Matches game or env
						(build_data.game_id IS NOT NULL AND b.game_id = build_data.game_id) OR
						(build_data.env_id IS NOT NULL AND b.env_id = build_data.env_id)
					",
					build_id,
					exclusive_tags_json,
				)
				.await?;

				// Determine builds to update
				let mut updates = Vec::new();
				for (build_id, current_tags_json) in all_builds {
					let current_tags: HashMap<String, serde_json::Value> =
						serde_json::from_value(current_tags_json)?;
					let current_tag_count = current_tags.len();

					// Remove values that match the new build's exclusive values
					let new_tags = current_tags
						.into_iter()
						.filter(|(key, value)| {
							!exclusive_tags_map
								.iter()
								.any(|(ex_key, ex_value)| ex_key == key && ex_value == value)
						})
						.collect::<HashMap<_, _>>();

					// Register update if tags changed
					if current_tag_count != new_tags.len() {
						let new_tags_json = util::serde::Raw::new(&new_tags)?;
						updates.push((build_id, new_tags_json));
					}
				}

				// Update builds
				let (build_ids, new_tags_jsons): (Vec<Uuid>, Vec<util::serde::Raw<_>>) =
					updates.into_iter().unzip();

				sql_execute!(
					[ctx, @tx tx]
					"
					UPDATE db_build.builds
					SET tags = updates.tags
					FROM unnest($1::uuid[], $2::jsonb[]) AS updates(build_id, tags)
					WHERE builds.build_id = updates.build_id
					",
					&build_ids,
					&new_tags_jsons,
				)
				.await?;
			}

			// Add tag to current build
			sql_execute!(
				[ctx, @tx tx]
				"
				UPDATE db_build.builds
				SET tags = tags || $2
				WHERE build_id = $1
				",
				build_id,
				tags_json,
			)
			.await?;

			Ok(())
		}
		.boxed()
	})
	.await?;

	Ok(Output {})
}
