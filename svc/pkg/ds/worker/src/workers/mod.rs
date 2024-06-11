pub mod nomad_monitor_alloc_plan;
pub mod nomad_monitor_alloc_update;
pub mod nomad_monitor_eval_update;

chirp_worker::workers![
	nomad_monitor_alloc_plan,
	nomad_monitor_alloc_update,
	nomad_monitor_eval_update
];

lazy_static::lazy_static! {
	pub static ref NEW_NOMAD_CONFIG: nomad_client::apis::configuration::Configuration =
		nomad_util::new_config_from_env().unwrap();
}
