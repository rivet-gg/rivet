provider "aws" {
	region = var.s3.region
	access_key = module.secrets.values["s3/terraform/key_id"]
	secret_key = module.secrets.values["s3/terraform/key"]

	# Config specifically for custom endpoints
	s3_use_path_style = true
	skip_credentials_validation = true
	skip_metadata_api_check = true
	skip_requesting_account_id = true

	endpoints {
		s3 = var.s3.endpoint_external
	}
}
