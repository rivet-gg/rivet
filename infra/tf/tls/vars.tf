variable "namespace" {
	type = string
}

# MARK: DNS
variable "dns_enabled" {
	type = bool
}

variable "domain_main" {
	type = string
	nullable = true
}

variable "domain_cdn" {
	type = string
	nullable = true
}

variable "domain_job" {
	type = string
	nullable = true
}

# MARK: Datacenters
variable "datacenters" {
	type = map(object({
		datacenter_id = string
	}))
}

# MARK: K8s
variable "kubeconfig_path" {
	type = string
}

# MARK: S3
variable "s3_providers" {
	type = map(object({
		endpoint_internal = string
		endpoint_external = string
		region = string
	}))
}
