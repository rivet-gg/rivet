provider "betteruptime" {
	api_token = module.betteruptime_secrets.values["better_uptime/token"]
}

module "betteruptime_secrets" {
	source = "../modules/secrets"
	keys = [
		"betteruptime/token",
		"rivet/api_status/token",
	]
}

resource "betteruptime_status_page" "status_page" {
	company_name = var.better_uptime.company_name
	company_url = var.better_uptime.company_url
	timezone = "UTC"
	subdomain = var.better_uptime.company_subdomain
	design = "v2"
	layout = "vertical"
	theme = "light"
}

output "status_page_domain" {
  description = "The domain of the Better Uptime status page"
  value       = betteruptime_status_page.status_page.domain
}

resource "betteruptime_status_page_section" "status_page_section" {
	status_page_id = betteruptime_status_page.status_page.id
	name = "Matchmaker"
	position = 0
}

resource "betteruptime_monitor" "monitor" {
	count = length(var.better_uptime_monitors)
	url = var.better_uptime_monitors[count.index].url
	monitor_type = "status"
	request_headers = [
		{
			name = "Authorization"
			value = "Bearer ${module.betteruptime_secrets.values["rivet/api_status/token"]}"
		}
	]
}

resource "betteruptime_status_page_resource" "status_page_resource" {
	count = length(var.better_uptime_monitors)
	public_name = var.better_uptime_monitors[count.index].public_name
	resource_id = betteruptime_monitor.monitor[count.index].id
	resource_type = "Monitor"
	status_page_id = betteruptime_status_page.status_page.id
	status_page_section_id = betteruptime_status_page_section.status_page_section.id
}
