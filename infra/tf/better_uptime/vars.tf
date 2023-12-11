# MARK: Better Uptime
variable "better_uptime_monitors" {
	type = list(object({
		url = string
		public_name = string
	}))
}

variable "better_uptime" {
	type = object({
		// The name of your company. This will be displayed on your status page
		// in the top left. This is required by Better Uptime.
		company_name = string
		// The URL of your company. This will be used on the status page to link
		// to your company's website. This is required by Better Uptime.
		company_url = string
		// The subdomain is the part of the public URL of your status page uses.
		//
		// Eg. <company_subdomain>.betteruptime.com.
		//
		// It needs to be unique across all of Better Uptime. This is required
		// by Better Uptime.
		company_subdomain = string
	})
}
