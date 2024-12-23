// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn parse_delete_error(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::DeleteOutput, crate::error::DeleteError> {
	let generic = crate::json_deser::parse_http_generic_error(response)
		.map_err(crate::error::DeleteError::unhandled)?;
	let error_code = match generic.code() {
		Some(code) => code,
		None => return Err(crate::error::DeleteError::unhandled(generic)),
	};

	let _error_message = generic.message().map(|msg| msg.to_owned());
	Err(match error_code {
		"InternalError" => crate::error::DeleteError {
			meta: generic,
			kind: crate::error::DeleteErrorKind::InternalError({
				#[allow(unused_mut)]
				let mut tmp = {
					#[allow(unused_mut)]
					let mut output = crate::error::internal_error::Builder::default();
					let _ = response;
					output =
						crate::json_deser::deser_structure_crate_error_internal_error_json_err(
							response.body().as_ref(),
							output,
						)
						.map_err(crate::error::DeleteError::unhandled)?;
					output.build()
				};
				if tmp.message.is_none() {
					tmp.message = _error_message;
				}
				tmp
			}),
		},
		"RateLimitError" => {
			crate::error::DeleteError {
				meta: generic,
				kind: crate::error::DeleteErrorKind::RateLimitError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::rate_limit_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_rate_limit_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"ForbiddenError" => {
			crate::error::DeleteError {
				meta: generic,
				kind: crate::error::DeleteErrorKind::ForbiddenError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::forbidden_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_forbidden_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"UnauthorizedError" => {
			crate::error::DeleteError {
				meta: generic,
				kind: crate::error::DeleteErrorKind::UnauthorizedError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::unauthorized_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_unauthorized_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"NotFoundError" => {
			crate::error::DeleteError {
				meta: generic,
				kind: crate::error::DeleteErrorKind::NotFoundError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::not_found_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_not_found_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"BadRequestError" => {
			crate::error::DeleteError {
				meta: generic,
				kind: crate::error::DeleteErrorKind::BadRequestError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::bad_request_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_bad_request_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		_ => crate::error::DeleteError::generic(generic),
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_delete_response(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::DeleteOutput, crate::error::DeleteError> {
	Ok({
		#[allow(unused_mut)]
		let mut output = crate::output::delete_output::Builder::default();
		let _ = response;
		output.build()
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_delete_batch_error(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::DeleteBatchOutput, crate::error::DeleteBatchError> {
	let generic = crate::json_deser::parse_http_generic_error(response)
		.map_err(crate::error::DeleteBatchError::unhandled)?;
	let error_code = match generic.code() {
		Some(code) => code,
		None => return Err(crate::error::DeleteBatchError::unhandled(generic)),
	};

	let _error_message = generic.message().map(|msg| msg.to_owned());
	Err(match error_code {
		"InternalError" => crate::error::DeleteBatchError {
			meta: generic,
			kind: crate::error::DeleteBatchErrorKind::InternalError({
				#[allow(unused_mut)]
				let mut tmp = {
					#[allow(unused_mut)]
					let mut output = crate::error::internal_error::Builder::default();
					let _ = response;
					output =
						crate::json_deser::deser_structure_crate_error_internal_error_json_err(
							response.body().as_ref(),
							output,
						)
						.map_err(crate::error::DeleteBatchError::unhandled)?;
					output.build()
				};
				if tmp.message.is_none() {
					tmp.message = _error_message;
				}
				tmp
			}),
		},
		"RateLimitError" => {
			crate::error::DeleteBatchError {
				meta: generic,
				kind: crate::error::DeleteBatchErrorKind::RateLimitError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::rate_limit_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_rate_limit_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"ForbiddenError" => {
			crate::error::DeleteBatchError {
				meta: generic,
				kind: crate::error::DeleteBatchErrorKind::ForbiddenError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::forbidden_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_forbidden_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"UnauthorizedError" => {
			crate::error::DeleteBatchError {
				meta: generic,
				kind: crate::error::DeleteBatchErrorKind::UnauthorizedError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::unauthorized_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_unauthorized_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"NotFoundError" => {
			crate::error::DeleteBatchError {
				meta: generic,
				kind: crate::error::DeleteBatchErrorKind::NotFoundError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::not_found_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_not_found_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"BadRequestError" => {
			crate::error::DeleteBatchError {
				meta: generic,
				kind: crate::error::DeleteBatchErrorKind::BadRequestError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::bad_request_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_bad_request_error_json_err(response.body().as_ref(), output).map_err(crate::error::DeleteBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		_ => crate::error::DeleteBatchError::generic(generic),
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_delete_batch_response(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::DeleteBatchOutput, crate::error::DeleteBatchError> {
	Ok({
		#[allow(unused_mut)]
		let mut output = crate::output::delete_batch_output::Builder::default();
		let _ = response;
		output.build()
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_get_error(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::GetOutput, crate::error::GetError> {
	let generic = crate::json_deser::parse_http_generic_error(response)
		.map_err(crate::error::GetError::unhandled)?;
	let error_code = match generic.code() {
		Some(code) => code,
		None => return Err(crate::error::GetError::unhandled(generic)),
	};

	let _error_message = generic.message().map(|msg| msg.to_owned());
	Err(match error_code {
		"InternalError" => crate::error::GetError {
			meta: generic,
			kind: crate::error::GetErrorKind::InternalError({
				#[allow(unused_mut)]
				let mut tmp = {
					#[allow(unused_mut)]
					let mut output = crate::error::internal_error::Builder::default();
					let _ = response;
					output =
						crate::json_deser::deser_structure_crate_error_internal_error_json_err(
							response.body().as_ref(),
							output,
						)
						.map_err(crate::error::GetError::unhandled)?;
					output.build()
				};
				if tmp.message.is_none() {
					tmp.message = _error_message;
				}
				tmp
			}),
		},
		"RateLimitError" => {
			crate::error::GetError {
				meta: generic,
				kind: crate::error::GetErrorKind::RateLimitError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::rate_limit_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_rate_limit_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"ForbiddenError" => {
			crate::error::GetError {
				meta: generic,
				kind: crate::error::GetErrorKind::ForbiddenError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::forbidden_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_forbidden_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"UnauthorizedError" => {
			crate::error::GetError {
				meta: generic,
				kind: crate::error::GetErrorKind::UnauthorizedError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::unauthorized_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_unauthorized_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"NotFoundError" => {
			crate::error::GetError {
				meta: generic,
				kind: crate::error::GetErrorKind::NotFoundError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::not_found_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_not_found_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"BadRequestError" => {
			crate::error::GetError {
				meta: generic,
				kind: crate::error::GetErrorKind::BadRequestError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::bad_request_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_bad_request_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		_ => crate::error::GetError::generic(generic),
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_get_response(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::GetOutput, crate::error::GetError> {
	Ok({
		#[allow(unused_mut)]
		let mut output = crate::output::get_output::Builder::default();
		let _ = response;
		output = crate::json_deser::deser_operation_crate_operation_get(
			response.body().as_ref(),
			output,
		)
		.map_err(crate::error::GetError::unhandled)?;
		output.build()
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_get_batch_error(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::GetBatchOutput, crate::error::GetBatchError> {
	let generic = crate::json_deser::parse_http_generic_error(response)
		.map_err(crate::error::GetBatchError::unhandled)?;
	let error_code = match generic.code() {
		Some(code) => code,
		None => return Err(crate::error::GetBatchError::unhandled(generic)),
	};

	let _error_message = generic.message().map(|msg| msg.to_owned());
	Err(match error_code {
		"InternalError" => crate::error::GetBatchError {
			meta: generic,
			kind: crate::error::GetBatchErrorKind::InternalError({
				#[allow(unused_mut)]
				let mut tmp = {
					#[allow(unused_mut)]
					let mut output = crate::error::internal_error::Builder::default();
					let _ = response;
					output =
						crate::json_deser::deser_structure_crate_error_internal_error_json_err(
							response.body().as_ref(),
							output,
						)
						.map_err(crate::error::GetBatchError::unhandled)?;
					output.build()
				};
				if tmp.message.is_none() {
					tmp.message = _error_message;
				}
				tmp
			}),
		},
		"RateLimitError" => {
			crate::error::GetBatchError {
				meta: generic,
				kind: crate::error::GetBatchErrorKind::RateLimitError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::rate_limit_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_rate_limit_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"ForbiddenError" => {
			crate::error::GetBatchError {
				meta: generic,
				kind: crate::error::GetBatchErrorKind::ForbiddenError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::forbidden_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_forbidden_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"UnauthorizedError" => {
			crate::error::GetBatchError {
				meta: generic,
				kind: crate::error::GetBatchErrorKind::UnauthorizedError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::unauthorized_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_unauthorized_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"NotFoundError" => {
			crate::error::GetBatchError {
				meta: generic,
				kind: crate::error::GetBatchErrorKind::NotFoundError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::not_found_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_not_found_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"BadRequestError" => {
			crate::error::GetBatchError {
				meta: generic,
				kind: crate::error::GetBatchErrorKind::BadRequestError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::bad_request_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_bad_request_error_json_err(response.body().as_ref(), output).map_err(crate::error::GetBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		_ => crate::error::GetBatchError::generic(generic),
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_get_batch_response(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::GetBatchOutput, crate::error::GetBatchError> {
	Ok({
		#[allow(unused_mut)]
		let mut output = crate::output::get_batch_output::Builder::default();
		let _ = response;
		output = crate::json_deser::deser_operation_crate_operation_get_batch(
			response.body().as_ref(),
			output,
		)
		.map_err(crate::error::GetBatchError::unhandled)?;
		output.build()
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_put_error(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::PutOutput, crate::error::PutError> {
	let generic = crate::json_deser::parse_http_generic_error(response)
		.map_err(crate::error::PutError::unhandled)?;
	let error_code = match generic.code() {
		Some(code) => code,
		None => return Err(crate::error::PutError::unhandled(generic)),
	};

	let _error_message = generic.message().map(|msg| msg.to_owned());
	Err(match error_code {
		"InternalError" => crate::error::PutError {
			meta: generic,
			kind: crate::error::PutErrorKind::InternalError({
				#[allow(unused_mut)]
				let mut tmp = {
					#[allow(unused_mut)]
					let mut output = crate::error::internal_error::Builder::default();
					let _ = response;
					output =
						crate::json_deser::deser_structure_crate_error_internal_error_json_err(
							response.body().as_ref(),
							output,
						)
						.map_err(crate::error::PutError::unhandled)?;
					output.build()
				};
				if tmp.message.is_none() {
					tmp.message = _error_message;
				}
				tmp
			}),
		},
		"RateLimitError" => {
			crate::error::PutError {
				meta: generic,
				kind: crate::error::PutErrorKind::RateLimitError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::rate_limit_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_rate_limit_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"ForbiddenError" => {
			crate::error::PutError {
				meta: generic,
				kind: crate::error::PutErrorKind::ForbiddenError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::forbidden_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_forbidden_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"UnauthorizedError" => {
			crate::error::PutError {
				meta: generic,
				kind: crate::error::PutErrorKind::UnauthorizedError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::unauthorized_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_unauthorized_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"NotFoundError" => {
			crate::error::PutError {
				meta: generic,
				kind: crate::error::PutErrorKind::NotFoundError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::not_found_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_not_found_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"BadRequestError" => {
			crate::error::PutError {
				meta: generic,
				kind: crate::error::PutErrorKind::BadRequestError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::bad_request_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_bad_request_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		_ => crate::error::PutError::generic(generic),
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_put_response(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::PutOutput, crate::error::PutError> {
	Ok({
		#[allow(unused_mut)]
		let mut output = crate::output::put_output::Builder::default();
		let _ = response;
		output.build()
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_put_batch_error(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::PutBatchOutput, crate::error::PutBatchError> {
	let generic = crate::json_deser::parse_http_generic_error(response)
		.map_err(crate::error::PutBatchError::unhandled)?;
	let error_code = match generic.code() {
		Some(code) => code,
		None => return Err(crate::error::PutBatchError::unhandled(generic)),
	};

	let _error_message = generic.message().map(|msg| msg.to_owned());
	Err(match error_code {
		"InternalError" => crate::error::PutBatchError {
			meta: generic,
			kind: crate::error::PutBatchErrorKind::InternalError({
				#[allow(unused_mut)]
				let mut tmp = {
					#[allow(unused_mut)]
					let mut output = crate::error::internal_error::Builder::default();
					let _ = response;
					output =
						crate::json_deser::deser_structure_crate_error_internal_error_json_err(
							response.body().as_ref(),
							output,
						)
						.map_err(crate::error::PutBatchError::unhandled)?;
					output.build()
				};
				if tmp.message.is_none() {
					tmp.message = _error_message;
				}
				tmp
			}),
		},
		"RateLimitError" => {
			crate::error::PutBatchError {
				meta: generic,
				kind: crate::error::PutBatchErrorKind::RateLimitError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::rate_limit_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_rate_limit_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"ForbiddenError" => {
			crate::error::PutBatchError {
				meta: generic,
				kind: crate::error::PutBatchErrorKind::ForbiddenError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::forbidden_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_forbidden_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"UnauthorizedError" => {
			crate::error::PutBatchError {
				meta: generic,
				kind: crate::error::PutBatchErrorKind::UnauthorizedError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::unauthorized_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_unauthorized_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"NotFoundError" => {
			crate::error::PutBatchError {
				meta: generic,
				kind: crate::error::PutBatchErrorKind::NotFoundError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::not_found_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_not_found_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"BadRequestError" => {
			crate::error::PutBatchError {
				meta: generic,
				kind: crate::error::PutBatchErrorKind::BadRequestError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::bad_request_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_bad_request_error_json_err(response.body().as_ref(), output).map_err(crate::error::PutBatchError::unhandled)?;
						output.build()
					};
					if tmp.message.is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		_ => crate::error::PutBatchError::generic(generic),
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_put_batch_response(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::PutBatchOutput, crate::error::PutBatchError> {
	Ok({
		#[allow(unused_mut)]
		let mut output = crate::output::put_batch_output::Builder::default();
		let _ = response;
		output.build()
	})
}
