mod cleanup;
mod create;
mod stop;
mod nomad_monitor_alloc_plan;
mod nomad_monitor_alloc_update;
mod nomad_monitor_alloc_eval_update;

chirp_worker::workers![
	cleanup,
	create,
	stop,
	nomad_monitor_alloc_plan,
	nomad_monitor_alloc_update,
	nomad_monitor_alloc_eval_update,
];
