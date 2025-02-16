use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend::pkg::{*};

use lazy_static::lazy_static;
use futures_util::{FutureExt, StreamExt, TryStreamExt};
use rand::{seq::IteratorRandom, Rng};
use serde_json::json;

lazy_static! {
	// Load adjectives from file
	static ref ADJECTIVES: Vec<&'static str> = include_str!("../../../adjectives.txt")
		.split('\n')
		.filter(|l| !l.is_empty())
		.map(|l| l.trim())
		.collect();
}

const UPLOAD_BATCH_SIZE: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
	pub user_id: Uuid,
	pub display_name: Option<String>,
	pub is_already_in_db: bool
}

#[workflow]
pub async fn user(ctx: &mut WorkflowCtx, input: &Input) -> GlobalResult<()> {
	if !input.is_already_in_db {
		let (display_name, _account_number) = ctx.activity(InsertDbInput {
			user_id: input.user_id,
			display_name: input.display_name.clone(),
		}).await?;

		ctx.activity(PublishCreationAnalyticsInput {
			user_id: input.user_id,
			display_name,
		}).await?;
	}

	ctx.msg(CreateComplete {})
		.tag("user_id", input.user_id)
		.send()
		.await?;

	ctx.repeat(|ctx| {
		let user_id = input.user_id;

		async move {
			match ctx.listen::<Main>().await? {
				Main::ChangedTeam(_) => {
					ctx.msg(Update {})
						.tag("user_id", user_id)
						.send()
						.await?;
				},
				Main::CreatedIdentity(sig) => {
					ctx.activity(LoopsContactCreateInput {
						user_id,
						identity: sig.identity
					}).await?;
					ctx.msg(Update {})
						.tag("user_id", user_id)
						.send()
						.await?;
				},
				Main::CompletedAvatarUpload(_) => {
					ctx.msg(Update {})
						.tag("user_id", user_id)
						.send()
						.await?;
				},
				Main::ToggledPendingDeletion(_) => {
					ctx.msg(Update {})
						.tag("user_id", user_id)
						.send()
						.await?;
				},
				Main::AdminSet(_) => {
					ctx.activity(AdminSetInput {
						user_id
					}).await?;

					ctx.msg(Update {})
						.tag("user_id", user_id)
						.send()
						.await?;
				},
				Main::ProfileSet(sig) => {
					let res = ctx.activity(ProfileSetInput {
						user_id,
						display_name: sig.display_name,
						account_number: sig.account_number,
						bio: sig.bio
					}).await?;

					ctx.msg(ProfileSetStatus { res: res.clone() })
							.tag("user_id", user_id)
							.send()
							.await?;
					
					if res.is_ok() {
						ctx.activity(PublishProfileSetAnalyticsInput {
							user_id
						}).await?;

						ctx.msg(Update {})
							.tag("user_id", user_id)
							.send()
							.await?;
					}
				},
				Main::Delete(_) => {
					return Ok(Loop::Break(()));
				},
			}

			Ok(Loop::Continue)
		}
		.boxed()
	}).await?;

	ctx.activity(DeleteIdentitiesInput {
		user_id: input.user_id,
	}).await?;

	ctx.activity(DeleteUploadsInput {
		user_id: input.user_id,
	}).await?;

	ctx.activity(RemoveFromTeamsInput {
		user_id: input.user_id,
	}).await?;

	ctx.activity(RedactUserRecordInput {
		user_id: input.user_id,
	}).await?;

	ctx.msg(DeleteComplete {})
		.tag("user_id", input.user_id)
		.send()
		.await?;

	ctx.msg(Update {})
		.tag("user_id", input.user_id)
		.send()
		.await?;

	ctx.activity(PublishDeletionAnalyticsInput {
		user_id: input.user_id,
	}).await?;

	Ok(())
}

// AdminSet
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct AdminSetInput {
	user_id: Uuid,
}

#[activity(AdminSetActivity)]
async fn admin_set(ctx: &ActivityCtx, input: &AdminSetInput) -> GlobalResult<()> {
	sql_execute!(
		[ctx]
		"
		UPDATE db_user.users
		SET
			is_admin = true
		WHERE user_id = $1
		",
		input.user_id,
	)
	.await?;

	Ok(())
}

// ProfileSet
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct ProfileSetInput {
	user_id: Uuid,
	display_name: Option<String>,
	account_number: Option<u32>,
	bio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum ProfileSetError {
	ValidationFailure,
	MissingParameters,
}

#[activity(ProfileSetActivity)]
async fn profile_set(ctx: &ActivityCtx, input: &ProfileSetInput) -> GlobalResult<Result<(), ProfileSetError>> {
	// Check if each component exists
	if input.display_name.is_none() && input.account_number.is_none() && input.bio.is_none() {
		return Ok(Err(ProfileSetError::MissingParameters));
	}

	let validation_res = ctx.op(crate::ops::profile_validate::Input {
		user_id: input.user_id,
		display_name: input.display_name.clone(),
		account_number: input.account_number,
		bio: input.bio.clone()
	})
	.await?;

	if !validation_res.errors.is_empty() {
		tracing::warn!(errors = ?validation_res.errors, "validation errors");

		return Ok(Err(ProfileSetError::ValidationFailure));
	}

	ctx.cache().purge("user", [input.user_id]).await?;

	sql_execute!(
		[ctx]
		"
		UPDATE db_user.users
		SET
			display_name = COALESCE($2, display_name),
			account_number = COALESCE($3, account_number),
			bio = COALESCE($4, bio)
		WHERE user_id = $1
		",
		input.user_id,
		&input.display_name,
		input.account_number.map(|x| x as i64),
		input.bio.as_ref().map(|x| util::format::biography(x))
	)
	.await?;

	Ok(Ok(()))
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct PublishProfileSetAnalyticsInput {
	user_id: Uuid
}

#[activity(PublishProfileSetAnalytics)]
async fn publish_profile_set_analytics(
	ctx: &ActivityCtx,
	input: &PublishProfileSetAnalyticsInput
) -> GlobalResult<()> {
	msg!([ctx] analytics::msg::event_create() {
		events: vec![
			analytics::msg::event_create::Event {
				event_id: Some(Uuid::new_v4().into()),
				name: "user.profile_set".into(),
				properties_json: Some(serde_json::to_string(&json!({
					"user_id": input.user_id.to_string()
				}))?),
				..Default::default()
			},
		],
	})
	.await?;

	Ok(())
}


// Creation
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct InsertDbInput {
	user_id: Uuid,
	display_name: Option<String>,
}

#[activity(InsertDb)]
#[max_retries = 5]
async fn insert_db(ctx: &ActivityCtx, input: &InsertDbInput) -> GlobalResult<(String, i64)> {
	let display_name = if let Some(display_name) = input.display_name.clone() {
		display_name
	} else {
		gen_display_name("Guest")
	};

	let account_number = gen_account_number();
	tracing::info!(%display_name, %account_number, "insert user attempt");

	sql_execute!(
		[ctx]
		"
		INSERT INTO db_user.users (
			user_id,
			display_name,
			account_number,
			avatar_id,
			join_ts
		)
		VALUES ($1, $2, $3, $4, $5)
		",
		input.user_id,
		&display_name,
		account_number,
		gen_avatar_id(),
		ctx.ts(),
	)
	.await?;

	Ok((display_name, account_number))
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct PublishCreationAnalyticsInput {
	user_id: Uuid,
	display_name: String,
}

#[activity(PublishCreationAnalytics)]
async fn publish_creation_analytics(ctx: &ActivityCtx, input: &PublishCreationAnalyticsInput) -> GlobalResult<()> {
	let properties_json = Some(serde_json::to_string(&json!({
		"user_id": input.user_id,
		"display_name": input.display_name,
	}))?);

	msg!([ctx] analytics::msg::event_create() {
		events: vec![
			analytics::msg::event_create::Event {
				event_id: Some(Uuid::new_v4().into()),
				name: "user.create".into(),
				properties_json: properties_json.clone(),
				..Default::default()
			},
			analytics::msg::event_create::Event {
				event_id: Some(Uuid::new_v4().into()),
				name: "user.profile_set".into(),
				properties_json,
				..Default::default()
			},
		],
	})
	.await?;

	Ok(())
}

// Deletion
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct DeleteIdentitiesInput {
	user_id: Uuid
}

#[activity(DeleteIdentities)]
async fn delete_identities(ctx: &ActivityCtx, input: &DeleteIdentitiesInput) -> GlobalResult<()> {
	ctx.op(crate::ops::identity::delete::Input {
		user_ids: vec![input.user_id]
	}).await?;

	Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct DeleteUploadsInput {
	user_id: Uuid
}

#[activity(DeleteUploads)]
async fn delete_uploads(ctx: &ActivityCtx, input: &DeleteUploadsInput) -> GlobalResult<()> {
	tracing::info!(user_id = %input.user_id, "removing uploads");
	let mut last_create_ts = 0;	
	loop {
		let uploads_res = op!([ctx] upload_list_for_user {
			user_ids: vec![input.user_id.into()],
			anchor: Some(last_create_ts),
			limit: UPLOAD_BATCH_SIZE as u32,
		})
		.await?;
		let user = unwrap!(uploads_res.users.first());

		let request_id = Uuid::new_v4();
		msg!([ctx] upload::msg::delete(request_id) -> upload::msg::delete_complete {
			request_id: Some(request_id.into()),
			upload_ids: user.upload_ids.clone(),
		})
		.await?;

		// Update last timestamp
		if let Some(anchor) = user.anchor {
			last_create_ts = anchor;
		}

		if user.upload_ids.len() < UPLOAD_BATCH_SIZE {
			break;
		}
	}

	Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct RemoveFromTeamsInput {
	user_id: Uuid
}

#[activity(RemoveFromTeams)]
async fn remove_from_teams(ctx: &ActivityCtx, input: &RemoveFromTeamsInput) -> GlobalResult<()> {
	tracing::info!(user_id = %input.user_id, "removing teams");

	let user_teams_res = ctx.op(crate::ops::team_list::Input {
		user_ids: vec![input.user_id],
	})
	.await?;
	let user_teams = unwrap!(user_teams_res.users.first());

	let teams_res = op!([ctx] team_get {
		team_ids: user_teams.teams
			.iter()
			.map(|member| Ok(member.team_id.into()))
			.collect::<GlobalResult<Vec<_>>>()?
	})
	.await?;

	// Filter out teams where the user is the owner
	let non_owner_teams = teams_res
		.teams
		.clone()
		.into_iter()
		.filter(|team| team.owner_user_id != Some(input.user_id.into()));
	futures_util::stream::iter(non_owner_teams)
		.map(|team| {
			let team_id_proto = team.team_id;

			async move {
				let team_id = unwrap!(team_id_proto).as_uuid();

				msg!([ctx] team::msg::member_remove(team_id, input.user_id) -> team::msg::member_remove_complete {
					user_id: Some(input.user_id.into()),
					team_id: team_id_proto,
					silent: false,
				})
				.await
				.map_err(Into::<GlobalError>::into)
			}
		})
		.buffer_unordered(32)
		.try_collect::<Vec<_>>()
		.await?;

	Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct RedactUserRecordInput {
	user_id: Uuid
}

#[activity(RedactUserRecord)]
async fn redact_user_record(ctx: &ActivityCtx, input: &RedactUserRecordInput) -> GlobalResult<()> {
	tracing::info!(user_id = %input.user_id, "removing user record");

	sql_execute!(
		[ctx]
		"
		UPDATE db_user.users
		SET
			display_name = $2,
			profile_id = NULL,
			bio = '',
			delete_complete_ts = $3
		WHERE user_id = $1
		",
		input.user_id,
		gen_deleted_user_display_name(),
		util::timestamp::now(),
	)
	.await?;

	ctx.cache().purge("user", [input.user_id]).await?;

	Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct PublishDeletionAnalyticsInput {
	user_id: Uuid
}

#[activity(PublishDeletionAnalytics)]
async fn publish_deletion_analytics(
	ctx: &ActivityCtx,
	input: &PublishDeletionAnalyticsInput
) -> GlobalResult<()> {
	msg!([ctx] analytics::msg::event_create() {
		events: vec![
			analytics::msg::event_create::Event {
				event_id: Some(Uuid::new_v4().into()),
				name: "user.delete".into(),
				properties_json: Some(serde_json::to_string(&json!({
					"deleted_user_id": input.user_id
				}))?),
				..Default::default()
			}
		],
	})
	.await?;

	Ok(())
}

// Identity Create
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct LoopsContactCreateInput {
	user_id: Uuid,
	identity: crate::types::identity::Identity,
}

#[activity(LoopsContactCreate)]
async fn loops_contact_create(
	ctx: &ActivityCtx,
	input: &LoopsContactCreateInput
) -> GlobalResult<()> {
	if let crate::types::identity::Kind::Email(email) = &input.identity.kind {
		ctx.op(loops::ops::create_contact::Input {
			user_id: input.user_id,
			email: email.email.clone(),
		}).await?;
	}

	Ok(())
}

#[message("user_create_complete")]
pub struct CreateComplete {}

#[message("user_update")]
pub struct Update {}

#[message("user_delete_complete")]
pub struct DeleteComplete {}

#[message("user_event")]
pub struct Event {}

#[message("user_profile_set_status")]
pub struct ProfileSetStatus {
	pub res: Result<(), ProfileSetError>,
}

#[signal("user_admin_set")]
pub struct AdminSet {}

#[signal("user_changed_team")]
pub struct ChangedTeam {}

#[signal("user_created_identity")]
pub struct CreatedIdentity {
	pub identity: crate::types::identity::Identity
}

#[signal("user_completed_avatar_upload")]
pub struct CompletedAvatarUpload {}

#[signal("user_toggled_pending_deletion")]
pub struct ToggledPendingDeletion {}

#[signal("user_profile_set")]
pub struct ProfileSet {
	pub display_name: Option<String>,
	pub account_number: Option<u32>,
	pub bio: Option<String>,
}

#[signal("user_delete")]
pub struct Delete {}

join_signal!(Main {
	ChangedTeam,
	CreatedIdentity,
	CompletedAvatarUpload,
	ToggledPendingDeletion,
	AdminSet,
	ProfileSet,
	Delete,
});

// Generates a display name with the format `{adjective:7}{space:1}{base:11}{space:1}{number:4}`
fn gen_display_name(base: impl std::fmt::Display) -> String {
	let base_str = format!("{}", base);

	let mut rand = rand::thread_rng();
	let adj = ADJECTIVES.iter().choose(&mut rand).unwrap_or(&"Unknown");

	format!(
		"{} {} {}",
		adj,
		base_str,
		std::iter::repeat_with(|| rand.gen_range(0..10))
			.map(|d| d.to_string())
			.take(4)
			.collect::<String>()
	)
}

// Generates a display name (for deleted users) with the format `Deleted User {alphanum:10}`
fn gen_deleted_user_display_name() -> String {
	format!(
		"Deleted User {}",
		rand::thread_rng()
			.sample_iter(rand::distributions::Alphanumeric)
			.map(char::from)
			.take(10)
			.collect::<String>()
	)
}


fn gen_account_number() -> i64 {
	rand::thread_rng().gen_range(1..10000)
}

fn gen_avatar_id() -> String {
	format!("avatar-{}", rand::thread_rng().gen_range(0..7))
}
