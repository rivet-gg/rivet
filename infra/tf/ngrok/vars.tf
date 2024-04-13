variable "namespace" {
    type = string
}

variable "ngrok_region" {
    type = string
    default = "us"
}

variable "ngrok_domain" {
    type = object({
        api = string
        minio = string
    })
}

variable "api_http_port" {
    type = number
}

variable "tunnel_port" {
    type = number
}

variable "minio_port" {
    type = number
}
