use aws_sdk_s3::error::{CreateBucketError, CreateBucketErrorKind};

use crate::{registry, Client, ClientError};

#[derive(Debug, thiserror::Error)]
pub enum ProvisionError {
	#[error("client error: {0}")]
	ClientError(#[from] ClientError),
	#[error("failed to create bucket: {0}")]
	CreateBucketError(String),
	#[error("missing env var: {0}")]
	MissingEnvVar(String),
}

/// Provisions all required S3 buckets.
pub async fn provision() -> Result<(), ProvisionError> {
	// Build client
	let region = std::env::var("S3_PROVISION_REGION")
		.map_err(|_| ProvisionError::MissingEnvVar("S3_PROVISION_REGION".into()))?;
	let access_key_id = std::env::var("S3_PROVISION_ACCESS_KEY_ID")
		.map_err(|_| ProvisionError::MissingEnvVar("S3_PROVISION_ACCESS_KEY_ID".into()))?;
	let secret_access_key = std::env::var("S3_PROVISION_SECRET_ACCESS_KEY")
		.map_err(|_| ProvisionError::MissingEnvVar("S3_PROVISION_SECRET_ACCESS_KEY".into()))?;
	let endpoint = std::env::var("S3_PROVISION_ENDPOINT")
		.map_err(|_| ProvisionError::MissingEnvVar("S3_PROVISION_ENDPOINT".into()))?;
	let client = Client::new("", &endpoint, &region, &access_key_id, &secret_access_key)?;

	// Provision buckets
	for bucket in registry::BUCKETS {
		let bucket_name_screaming = bucket.name.to_uppercase().replace("-", "_");
		let var_name = format!("S3_BUCKET_{bucket_name_screaming}");
		let bucket_name =
			std::env::var(&var_name).map_err(|_| ProvisionError::MissingEnvVar(var_name))?;

		match client.create_bucket().bucket(&bucket_name).send().await {
			Ok(_) => tracing::info!(bucket = ?bucket_name, "bucket created"),
			Err(err) => {
				if let aws_sdk_s3::types::SdkError::ServiceError(service_err) = &err {
					if let CreateBucketError {
						kind: CreateBucketErrorKind::BucketAlreadyExists(_),
						..
					} = service_err.err()
					{
						tracing::info!(bucket = ?bucket_name, "bucket already exists");
						continue;
					}
				}
				return Err(ProvisionError::CreateBucketError(err.to_string()));
			}
		}
	}

	Ok(())
}
