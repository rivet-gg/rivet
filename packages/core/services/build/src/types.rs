use std::collections::HashMap;

use chirp_workflow::prelude::*;
use rivet_api::models;
use rivet_convert::{ApiFrom, ApiTryFrom};
use strum::FromRepr;

// NOTE: Do not change the serde case of this or else it will break workflow hashes
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, FromRepr)]
pub enum BuildKind {
	DockerImage = 0,
	OciBundle = 1,
	JavaScript = 2,
}

impl ApiFrom<models::BuildsKind> for BuildKind {
	fn api_from(value: models::BuildsKind) -> BuildKind {
		match value {
			models::BuildsKind::DockerImage => BuildKind::DockerImage,
			models::BuildsKind::OciBundle => BuildKind::OciBundle,
			models::BuildsKind::Javascript => BuildKind::JavaScript,
		}
	}
}

// NOTE: Do not change the serde case of this or else it will break workflow hashes
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, FromRepr)]
pub enum BuildCompression {
	None = 0,
	Lz4 = 1,
}

impl ApiFrom<models::BuildsCompression> for BuildCompression {
	fn api_from(value: models::BuildsCompression) -> BuildCompression {
		match value {
			models::BuildsCompression::None => BuildCompression::None,
			models::BuildsCompression::Lz4 => BuildCompression::Lz4,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
	pub build_id: Uuid,
	pub game_id: Option<Uuid>,
	pub env_id: Option<Uuid>,
	pub upload_id: Uuid,
	pub display_name: String,
	pub image_tag: String,
	pub create_ts: i64,
	pub kind: BuildKind,
	pub compression: BuildCompression,
	pub allocation_type: BuildAllocationType,
	pub allocation_total_slots: u32,
	pub resources: Option<BuildResources>,
	pub tags: HashMap<String, String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, FromRepr)]
#[serde(rename_all = "snake_case")]
pub enum BuildAllocationType {
	None = 0,
	Single = 1,
	Multi = 2,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct BuildResources {
	pub cpu_millicores: u32,
	pub memory_mib: u32,
}

impl ApiTryFrom<models::BuildsResources> for BuildResources {
	type Error = GlobalError;

	fn api_try_from(value: models::BuildsResources) -> GlobalResult<Self> {
		ensure_with!(
			value.cpu >= 0,
			API_BAD_BODY,
			reason = "`resources.cpu` must be positive"
		);
		ensure_with!(
			value.memory >= 0,
			API_BAD_BODY,
			reason = "`resources.memory` must be positive"
		);

		Ok(BuildResources {
			cpu_millicores: value.cpu.try_into()?,
			memory_mib: value.memory.try_into()?,
		})
	}
}

impl ApiTryFrom<BuildResources> for models::BuildsResources {
	type Error = GlobalError;

	fn api_try_from(value: BuildResources) -> GlobalResult<Self> {
		Ok(models::BuildsResources {
			cpu: value.cpu_millicores.try_into()?,
			memory: value.memory_mib.try_into()?,
		})
	}
}

// TODO: Move to upload pkg when its converted to new ops
pub mod upload {
	use std::convert::TryInto;

	use chirp_workflow::prelude::*;
	use rivet_api::models;
	use rivet_convert::ApiTryFrom;
	use rivet_operation::prelude::proto::backend;

	#[derive(Debug)]
	pub struct PrepareFile {
		pub path: String,
		pub mime: Option<String>,
		pub content_length: u64,
		pub multipart: bool,
	}

	impl ApiTryFrom<models::UploadPrepareFile> for PrepareFile {
		type Error = GlobalError;

		fn api_try_from(value: models::UploadPrepareFile) -> GlobalResult<Self> {
			Ok(PrepareFile {
				path: value.path,
				mime: value.content_type,
				content_length: value.content_length.try_into()?,
				multipart: false,
			})
		}
	}

	#[derive(Debug)]
	pub struct PresignedUploadRequest {
		pub path: String,
		pub url: String,
		pub part_number: u32,
		pub byte_offset: u64,
		pub content_length: u64,
	}

	impl From<backend::upload::PresignedUploadRequest> for PresignedUploadRequest {
		fn from(value: backend::upload::PresignedUploadRequest) -> Self {
			PresignedUploadRequest {
				path: value.path,
				url: value.url,
				part_number: value.part_number,
				byte_offset: value.byte_offset,
				content_length: value.content_length,
			}
		}
	}

	impl ApiTryFrom<PresignedUploadRequest> for models::UploadPresignedRequest {
		type Error = GlobalError;

		fn api_try_from(value: PresignedUploadRequest) -> GlobalResult<Self> {
			Ok(models::UploadPresignedRequest {
				path: value.path,
				url: value.url,
				byte_offset: value.byte_offset.try_into()?,
				content_length: value.content_length.try_into()?,
			})
		}
	}
}
