provider "betteruptime" {
	api_token = module.betteruptime_secrets.values["betteruptime/token"]
}

module "betteruptime_secrets" {
	source = "../modules/secrets"
	keys = [
		"betteruptime/token",
		"rivet/api_status/token",
	]
}

resource "betteruptime_status_page" "status_page" {
	count = length(var.betteruptime_monitors) > 0 ? 1 : 0
	
	company_name = var.betteruptime_company.company_name
	company_url = var.betteruptime_company.company_url
	timezone = "UTC"
	subdomain = var.betteruptime_company.company_subdomain
	design = "v2"
	layout = "vertical"
	theme = "light"
}

resource "betteruptime_status_page_section" "status_page_section" {
	count = length(var.betteruptime_monitors) > 0 ? 1 : 0

	status_page_id = betteruptime_status_page.status_page[0].id
	name = "Matchmaker"
	position = 0
}

resource "betteruptime_monitor" "monitor" {
	count = length(var.betteruptime_monitors)
	url = var.betteruptime_monitors[count.index].url
	monitor_type = "status"
	request_headers = [
		{
			name = "Authorization"
			value = "Bearer ${module.betteruptime_secrets.values["rivet/api_status/token"]}"
		}
	]
}

resource "betteruptime_status_page_resource" "status_page_resource" {
	count = length(var.betteruptime_monitors)
	public_name = var.betteruptime_monitors[count.index].public_name
	resource_id = betteruptime_monitor.monitor[count.index].id
	resource_type = "Monitor"
	status_page_id = betteruptime_status_page.status_page.id
	status_page_section_id = betteruptime_status_page_section.status_page_section.id
}
