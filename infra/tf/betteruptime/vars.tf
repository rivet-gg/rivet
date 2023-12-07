# MARK: BetterUptime
variable "betteruptime_monitors" {
	type = list(object({
		url = string
		public_name = string
	}))
}

variable "betteruptime" {
	type = object({
		company_name = string
		company_url = string
		company_subdomain = string
	})
}
