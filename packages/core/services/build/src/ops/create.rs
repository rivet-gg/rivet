use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;

const MAX_UPLOAD_SIZE: u64 = util::file_size::gigabytes(8);
const MAX_JS_BUILD_UPLOAD_SIZE: u64 = util::file_size::megabytes(10);
use crate::{
	types::{
		upload::PrepareFile, upload::PresignedUploadRequest, BuildCompression, BuildKind,
		BuildNetworkMode, BuildRuntime,
	},
	utils,
};

#[derive(Debug)]
pub struct Input {
	pub owner: Owner,
	pub display_name: String,
	pub content: Content,
	pub kind: BuildKind,
	pub compression: BuildCompression,
	pub runtime: Option<BuildRuntime>,
}

#[derive(Debug)]
pub enum Owner {
	Game(Uuid),
	Env(Uuid),
}

#[derive(Debug)]
pub enum Content {
	New {
		image_file: PrepareFile,
		image_tag: String,
	},
	Default {
		build_kind: String,
	},
}

#[derive(Debug)]
pub struct Output {
	pub build_id: Uuid,
	pub upload_id: Uuid,
	pub presigned_requests: Vec<PresignedUploadRequest>,
}

#[operation]
pub async fn build_create(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	let dc_id = ctx.config().server()?.rivet.edge()?.datacenter_id;

	ensure_with!(
		util::check::display_name_long(&input.display_name),
		BUILD_INVALID,
		reason = "invalid display name"
	);

	let tier_res = ctx
		.op(tier::ops::list::Input {
			datacenter_ids: vec![dc_id],
			pegboard: true,
		})
		.await?;
	let tier_dc = unwrap!(tier_res.datacenters.into_iter().next());

	// Validate game exists
	let (game_id, env_id) = match input.owner {
		Owner::Game(game_id) => {
			let game_res = op!([ctx] game_get {
				game_ids: vec![game_id.into()],
			})
			.await?;
			let game = game_res.games.first();
			ensure!(game.is_some(), "game not found");

			(Some(game_id), None)
		}
		Owner::Env(env_id) => {
			let env_res = op!([ctx] game_namespace_get {
				namespace_ids: vec![env_id.into()],
			})
			.await?;
			let env = env_res.namespaces.first();
			ensure!(env.is_some(), "game not found");

			(None, Some(env_id))
		}
	};

	// Validate runtime config
	if let Some(runtime) = &input.runtime {
		if let BuildRuntime::Actor { resources, .. } = &runtime {
			// Find any tier that has more CPU and memory than the requested resources
			let has_tier = tier_dc.tiers.iter().any(|t| {
				t.cpu_millicores >= resources.cpu_millicores && t.memory >= resources.memory_mib
			});

			ensure_with!(
				has_tier,
				BUILD_INVALID,
				reason = "Too many resources allocated."
			);
		}

		ensure_with!(
			runtime.args().len() <= 64,
			BUILD_INVALID,
			reason = "Too many arguments (max 64)."
		);

		for (i, arg) in runtime.args().iter().enumerate() {
			ensure_with!(
				arg.len() <= 256,
				BUILD_INVALID,
				reason = format!("runtime.args[{i}]: Argument too large (max 256 bytes).")
			);
		}

		ensure_with!(
			runtime.environment().len() <= 64,
			BUILD_INVALID,
			reason = "Too many environment variables (max 64)."
		);

		for (k, v) in runtime.environment() {
			ensure_with!(
				k.len() <= 256,
				BUILD_INVALID,
				reason = format!(
					"runtime.environment[{:?}]: Key too large (max 256 bytes).",
					util::safe_slice(k, 0, 256),
				)
			);

			ensure_with!(
				v.len() <= 1024,
				BUILD_INVALID,
				reason = format!("runtime.environment[{k:?}]: Value too large (max 1024 bytes).")
			);
		}

		ensure_with!(
			runtime.ports().len() <= 8,
			BUILD_INVALID,
			reason = "Too many ports (max 8)."
		);

		for (name, port) in runtime.ports() {
			ensure_with!(
				name.len() <= 16,
				BUILD_INVALID,
				reason = format!(
					"runtime.ports[{:?}]: Port name too large (max 16 bytes).",
					util::safe_slice(name, 0, 16),
				)
			);

			match runtime.network_mode() {
				BuildNetworkMode::Bridge => {
					// NOTE: Temporary validation until we implement bridge networking for isolates
					if let BuildKind::JavaScript = input.kind {
						ensure_with!(
							port.internal_port.is_none(),
							BUILD_INVALID,
							reason = format!(
								"runtime.ports[{name:?}].internal_port: Must be null when `network.mode` = \"bridge\" and using a JS build."
							)
						);
					}
				}
				BuildNetworkMode::Host => {
					ensure_with!(
						port.internal_port.is_none(),
						BUILD_INVALID,
						reason = format!(
							"runtime.ports[{name:?}].internal_port: Must be null when `network.mode` = \"host\"."
						)
					);
				}
			}
		}
	}

	let (image_tag, upload_id, presigned_requests) = match &input.content {
		Content::Default { build_kind } => {
			let default_build_row = sql_fetch_optional!(
				[ctx, (String, Uuid)]
				"SELECT image_tag, upload_id FROM db_build.default_builds WHERE kind = $1",
				build_kind,
			)
			.await?;

			let (image_tag, upload_id) =
				unwrap!(default_build_row, "default build missing: {build_kind}");

			(image_tag, upload_id, Vec::new())
		}
		Content::New {
			image_file,
			image_tag,
		} => {
			let tag_split = image_tag.split_once(':');
			let (tag_base, tag) = unwrap_ref!(tag_split, "missing separator in image tag");
			ensure!(
				util::check::docker_ident(tag_base),
				"invalid image tag base"
			);
			ensure!(util::check::docker_ident(tag), "invalid tag");

			let max_upload_size = match input.kind {
				BuildKind::DockerImage | BuildKind::OciBundle => MAX_UPLOAD_SIZE,
				BuildKind::JavaScript => MAX_JS_BUILD_UPLOAD_SIZE,
			};
			ensure_with!(
				image_file.content_length < max_upload_size,
				UPLOAD_TOO_LARGE
			);

			// Check if build is unique
			let (build_exists,) = sql_fetch_one!(
				[ctx, (bool,)]
				"SELECT EXISTS (SELECT 1 FROM db_build.builds WHERE image_tag = $1)",
				image_tag,
			)
			.await?;
			if build_exists {
				bail!("build image tag not unique: {image_tag:?}");
			} else {
				tracing::debug!(?image_tag, "build image is unique");
			}

			// Create the upload
			let file_name = utils::file_name(input.kind, input.compression);
			let upload_prepare_res = op!([ctx] upload_prepare {
				bucket: "bucket-build".into(),
				files: vec![
					backend::upload::PrepareFile {
						path: file_name,
						content_length: image_file.content_length,
						..Default::default()
					},
				],
			})
			.await?;
			let upload_id = unwrap_ref!(upload_prepare_res.upload_id).as_uuid();

			(
				image_tag.clone(),
				upload_id,
				upload_prepare_res.presigned_requests.clone(),
			)
		}
	};

	// Create build
	let build_id = Uuid::new_v4();
	sql_execute!(
		[ctx]
		"
		INSERT INTO
			db_build.builds (
				build_id,
				game_id,
				env_id,
				upload_id,
				display_name,
				image_tag,
				create_ts,
				kind,
				compression,
				runtime
			)
		VALUES
			($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
		",
		build_id,
		game_id,
		env_id,
		upload_id,
		&input.display_name,
		&image_tag,
		ctx.ts(),
		input.kind as i32,
		input.compression as i32,
		sqlx::types::Json(&input.runtime),
	)
	.await?;

	Ok(Output {
		build_id,
		upload_id,
		presigned_requests: presigned_requests.into_iter().map(Into::into).collect(),
	})
}
