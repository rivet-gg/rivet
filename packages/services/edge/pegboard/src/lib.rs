#[cfg(feature = "chirp")]
use chirp_workflow::prelude::*;

#[cfg(feature = "workflows")]
pub mod workflows;

#[cfg(feature = "workflows")]
pub fn registry() -> WorkflowResult<Registry> {
	use workflows::*;

	let mut registry = Registry::new();
	registry.register_workflow::<client::Workflow>()?;
	registry.register_workflow::<actor::Workflow>()?;

	Ok(registry)
}
