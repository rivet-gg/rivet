resource "null_resource" "server_gc" {
	depends_on = [
		module.servers,
	]

	triggers = {
		servers = join(",", keys(var.servers))
	}
	
	provisioner "local-exec" {
		interpreter = ["/usr/bin/env", "-i", "bash", "-c"]
		command = "salt-key -L | grep -v ':' | grep -vx \"${join("\\|", keys(var.servers))}\" | xargs -L 1 salt-key -d"
	}
}
