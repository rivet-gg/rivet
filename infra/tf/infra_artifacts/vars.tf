variable "namespace" {
	type = string
}

# MARK: S3
variable "s3" {
	type = object({
		provider = string
		endpoint_internal = string
		endpoint_external = string
		region = string
	})
}

