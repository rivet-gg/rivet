use api_helper::{define_router, util::CorsConfigBuilder};
use hyper::{Body, Request, Response};
use rivet_api::models;
use rivet_operation::prelude::*;
use serde::Deserialize;

pub mod actors;
pub mod containers;

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalQuery {
	/// Slug of the project.
	///
	/// If provided, `environment` must also be provided.
	pub project: Option<String>,

	/// Slug of the environment.
	pub environment: Option<String>,
}

impl GlobalQuery {
	/// Returns both the project and environment.
	///
	/// Validates that the project can only be specified with the environment.
	pub fn project_and_env(&self) -> GlobalResult<(Option<&str>, Option<&str>)> {
		if let Some(environment) = &self.environment {
			Ok((self.project.as_deref(), Some(environment)))
		} else {
			// Don't allow just the project
			if self.project.is_some() {
				bail_with!(
					API_BAD_QUERY,
					parameter = "project",
					error = "Must provide both `project` and `environment` query together."
				)
			} else {
				Ok((None, None))
			}
		}
	}
}

define_router! {
	db_driver: chirp_workflow::db::DatabaseFdbSqliteNats,
	cors: |_config| CorsConfigBuilder::public().build(),
	routes: {
		"v2" / "actors": {
			GET: actors::list_actors(
				query: actors::ListQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
			POST: actors::create(
				query: actors::GlobalEndpointTypeQuery,
				body: models::ActorsCreateActorRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"v2" / "actors" / "upgrade": {
			POST: actors::upgrade_all(
				query: GlobalQuery,
				body: models::ActorsUpgradeAllActorsRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"v2" / "actors" / util::Id: {
			GET: actors::get(
				query: actors::GlobalEndpointTypeQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 60_000, bucket: duration::minutes(1) },
					],
				},

			),
			DELETE: actors::destroy(
				query: actors::DeleteQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"v2" / "actors" / util::Id / "upgrade": {
			POST: actors::upgrade(
				query: GlobalQuery,
				body: models::ActorsUpgradeActorRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"v1" / "containers": {
			GET: containers::list_containers(
				query: containers::ListQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
			POST: containers::create(
				query: containers::GlobalEndpointTypeQuery,
				body: models::ContainersCreateContainerRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"v1" / "containers" / "upgrade": {
			POST: containers::upgrade_all(
				query: GlobalQuery,
				body: models::ContainersUpgradeAllContainersRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"v1" / "containers" / util::Id: {
			GET: containers::get(
				query: containers::GlobalEndpointTypeQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 60_000, bucket: duration::minutes(1) },
					],
				},

			),
			DELETE: containers::destroy(
				query: containers::DeleteQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"v1" / "containers" / util::Id / "upgrade": {
			POST: containers::upgrade(
				query: GlobalQuery,
				body: models::ContainersUpgradeContainerRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},
	},

	mounts: [
		{
			path: OldRouter,
		},
		{
			path: OldRouter,
			prefix: "/v1"
		},
	],
}

define_router! {
	name: OldRouter,
	db_driver: chirp_workflow::db::DatabaseFdbSqliteNats,
	cors: |_config| CorsConfigBuilder::public().build(),
	routes: {
		"actors": {
			GET: actors::v1::list_actors(
				query: actors::v1::ListQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
			POST: actors::v1::create(
				query: actors::v1::GlobalEndpointTypeQuery,
				body: models::ActorsV1CreateActorRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"actors" / "upgrade": {
			POST: actors::v1::upgrade_all(
				query: GlobalQuery,
				body: models::ActorsV1UpgradeAllActorsRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"actors" / Uuid: {
			GET: actors::v1::get(
				query: actors::v1::GlobalEndpointTypeQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 60_000, bucket: duration::minutes(1) },
					],
				},

			),
			DELETE: actors::v1::destroy(
				query: actors::v1::DeleteQuery,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},

		"actors" / Uuid / "upgrade": {
			POST: actors::v1::upgrade(
				query: GlobalQuery,
				body: models::ActorsV1UpgradeActorRequest,
				opt_auth: true,
				rate_limit: {
					buckets: [
						{ count: 10_000, bucket: duration::minutes(1) },
					],
				},
			),
		},
	},
}
