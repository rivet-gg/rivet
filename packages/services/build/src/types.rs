use chirp_workflow::prelude::*;
use std::collections::HashMap;
use strum::FromRepr;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, FromRepr)]
pub enum BuildKind {
	DockerImage = 0,
	OciBundle = 1,
	JavaScript = 2,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, FromRepr)]
pub enum BuildCompression {
	None = 0,
	Lz4 = 1,
}

#[derive(Debug)]
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
	pub tags: HashMap<String, Option<String>>,
}

// TODO: Move to upload pkg when its converted to new ops
mod upload {
	use chirp_workflow::prelude::*;
	use rivet_operation::prelude::proto::backend;

	#[derive(Debug)]
	pub struct PrepareFile {
		pub path: String,
		pub mime: Option<String>,
		pub content_length: u64,
		pub multipart: bool,
	}

	#[derive(Debug)]
	pub struct PresignedUploadRequest {
		pub path: String,
		pub url: String,
		pub part_number: u32,
		pub byte_offset: u64,
		pub content_length: u64,
	}

	impl From<PresignedUploadRequest> for backend::upload::PresignedUploadRequest {
		fn from(value: PresignedUploadRequest) -> Self {
			backend::upload::PresignedUploadRequest {
				path: value.path,
				url: value.url,
				part_number: value.part_number,
				byte_offset: value.byte_offset,
				content_length: value.content_length,
			}
		}
	}
}
