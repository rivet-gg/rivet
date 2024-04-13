terraform {
    required_providers {
        ngrok = {
            source = "ngrok/ngrok"
            version = "0.2.0"
        }
        docker = {
            source = "kreuzwerker/docker"
            version = "3.0.2"
        }
    }
}

module "secrets" {
	source = "../modules/secrets"

	keys = [
		"ngrok/api_key",
		"ngrok/auth_token",
	]
}

resource "ngrok_reserved_addr" "tunnel" {
    description = "Rivet ${var.namespace} Tunnel"
    region = var.ngrok_region
}

resource "local_file" "ngrok_config_file" {
    filename = "/etc/rivet/ngrok.yaml"
    content = yamlencode({
        version = 2
        authtoken  = module.secrets.values["ngrok/auth_token"]
        tunnels = merge(
            {
                api = {
                    proto = "http"
                    addr = var.api_http_port
                    domain = var.ngrok_domain.api
                }
                tunnel = {
                    proto = "tcp"
                    addr = var.tunnel_port
                    remote_addr = ngrok_reserved_addr.tunnel.addr
                }
            },
            var.minio_port != null ? {
                minio = {
                    proto = "http"
                    addr = var.minio_port
                    domain = var.ngrok_domain.minio
                }
            } : {}
        )
    })
}

resource "docker_container" "ngrok" {
    name = "rivet-ngrok"
    image = "ngrok/ngrok:latest"
    restart = "unless-stopped"
    network_mode = "host"
    command = [
        "start",
        "--all",
        "--config",
        "/etc/ngrok.yaml"
    ]
    volumes {
        container_path = "/etc/ngrok.yaml"
        host_path = local_file.ngrok_config_file.filename
    }
}
