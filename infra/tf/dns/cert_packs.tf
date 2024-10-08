locals {
	# Required if:
	#
	# - Backend is enabled so requires access to `*.backend.{domain_main}`
 	# - Main domain is not at the root of the zone, we need to provide a cert pack for the domain.
	# - Using the old `{service}.api.{domain}` format, which requires two levels of subdomains.
 	needs_main_cert_pack = var.backend_enabled || var.dns_deprecated_subdomains || data.cloudflare_zone.main.name != var.domain_main

 	# If CDN is not at the root of the zone, we need to provide a cert pack for the CDN.
 	# 
 	# If the CDN domain is already at the root of the zone, then Cloudflare exposes a cert back by default and we don't need to create a new one.
 	needs_cdn_cert_pack = data.cloudflare_zone.cdn.name != var.domain_cdn

	# Should be `lets_encrypt` to be consistent with job node certs, change to `google` if experiencing rate
	# limits
	certificate_authority = "lets_encrypt"
}

# Allow Cloudflare to serve TLS requests at the edge for our wildcard
# subdomains.
#
# This requires paying money for these certs.
resource "cloudflare_certificate_pack" "main" {
	count = local.needs_main_cert_pack ? 1 : 0
	
	lifecycle {
		create_before_destroy = true
	}

	certificate_authority = local.certificate_authority
	# The certificate must include the root domain in it.
	#
	# We convert to set then back to list to remove potential duplicates of the root zone.
	hosts = sort(tolist(toset(
		flatten([
			[
				data.cloudflare_zone.main.name,
				var.domain_main,
				"*.${var.domain_main}",
				# TODO: Only if we use deprecated subdomains
				"*.api.${var.domain_main}",
			],
			var.backend_enabled ? [
				"*.backend.${var.domain_main}",
			] : []
		])
	)))
	type = "advanced"
	validation_method = "txt"
	validity_days = 90
	zone_id = local.cloudflare_zone_id_main
	wait_for_active_status = true
}

moved {
	from = cloudflare_certificate_pack.rivet_gg
	to = cloudflare_certificate_pack.main
}

# Allow for Cloudflare to proxy wildcard requests to this namespace.
resource "cloudflare_certificate_pack" "cdn" {
	count = local.needs_cdn_cert_pack ? 1 : 0

	lifecycle {
		create_before_destroy = true
	}

	certificate_authority = local.certificate_authority
	# The certificate must include the root domain in it.
	#
	# We convert to set then back to list to remove potential duplicates of the root zoon.
	hosts = sort(tolist(toset([
		data.cloudflare_zone.cdn.name,
        var.domain_cdn,
        "*.${var.domain_cdn}"
    ])))
	type = "advanced"
	validation_method = "txt"
	validity_days = 90
	zone_id = local.cloudflare_zone_id_cdn
	wait_for_active_status = true
}

moved {
	from = cloudflare_certificate_pack.rivet_game
	to = cloudflare_certificate_pack.cdn
}
