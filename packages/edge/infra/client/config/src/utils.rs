pub const RIVET_CONTAINER_PREFIX: &str = "rivet-";

pub fn format_container_id(actor_id: &str, generation: u32) -> String {
	format!("{RIVET_CONTAINER_PREFIX}{actor_id}-{generation}")
}
