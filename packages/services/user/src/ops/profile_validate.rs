use chirp_workflow::prelude::*;
use rivet_operation::prelude::common;

#[derive(Debug)]
pub struct Input {
    pub user_id: Uuid,
    pub display_name: Option<String>,
	pub account_number: Option<u32>,
	pub bio: Option<String>,
}

#[derive(Debug)]
pub struct Output {
    pub errors: Vec<common::ValidationError>,
}


#[operation]
pub async fn profile_validate(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
	let mut errors = Vec::new();

	// Validate display name
	if let Some(display_name) = &input.display_name {
		if display_name.is_empty() {
			errors.push(util::err_path!["display-name", "too-short"]);
		} else if display_name.len() > util::check::MAX_DISPLAY_NAME_LEN {
			errors.push(util::err_path!["display-name", "too-long"]);
		}

		if !util::check::display_name(display_name) {
			errors.push(util::err_path!["display-name", "invalid"]);
		}
	}

	// Validate account number
	if let Some(account_number) = &input.account_number {
		if *account_number < 1 || *account_number > 9999 {
			errors.push(util::err_path!["account-number-invalid"]);
		}
	}

	// Validate biography
	if let Some(bio) = &input.bio {
		if bio.len() > util::check::MAX_BIOGRAPHY_LEN {
			errors.push(util::err_path!["bio", "too-long"]);
		}

		if !util::check::biography(bio) {
			errors.push(util::err_path!["bio", "invalid"]);
		}
	}

	// Only validate handle uniqueness if at least one of the two handle components is given
	if input.display_name.is_some() || input.account_number.is_some() {
		// If either the display name or account number are missing, fetch them from the given user
		let (display_name, account_number) =
			if input.display_name.is_none() || input.account_number.is_none() {
				let users_res = op!([ctx] user_get {
					user_ids: vec![input.user_id.into()],
				})
				.await?;

				let user = users_res.users.first();
				let user = unwrap_ref!(user, "user not found");

				(
					input.display_name
						.clone()
						.unwrap_or(user.display_name.clone()),
					input.account_number.unwrap_or(user.account_number),
				)
			} else {
				(
					unwrap_ref!(input.display_name).clone(),
					*unwrap_ref!(input.account_number),
				)
			};

		// Find user by handle
		let (user_exists,) = sql_fetch_one!(
			[ctx, (bool,)]
			"
			SELECT EXISTS (
				SELECT 1
				FROM db_user.users
				WHERE display_name = $1 and account_number = $2
			)
			",
			display_name,
			account_number as i64,
		)
		.await?;

		// Validate handle uniqueness
		if user_exists {
			errors.push(util::err_path!["handle-not-unique"]);
		}
	}

	Ok(Output {
		errors: errors
			.into_iter()
			.map(|path| common::ValidationError { path })
			.collect::<Vec<_>>(),
	})
}