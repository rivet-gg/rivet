use chirp_workflow::prelude::*;

pub mod ops;
pub mod workflows;
pub mod types;
pub mod route;

pub fn registry() -> WorkflowResult<Registry> {
	use workflows::*;

	let mut registry = Registry::new();
	registry.register_workflow::<user::Workflow>()?;

	Ok(registry)
}
