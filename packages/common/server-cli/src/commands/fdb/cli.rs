use std::{
	result::Result::{Err, Ok},
	time::Duration,
};

use clap::{Parser, ValueEnum};
use foundationdb::{self as fdb, options::StreamingMode};
use futures_util::TryStreamExt;
use rivet_pools::FdbPool;
use rivet_term::console::style;

use crate::util::{
	fdb::{ListStyle, SimpleTuple, SimpleValue},
	format::indent_string,
};

// TODO: Tab completion
#[derive(Parser)]
#[command(name = "")]
pub enum SubCommand {
	/// Change current key.
	#[command(name = "cd")]
	ChangeKey {
		/// Key path to change to. Supports relative key paths.
		key: String,
	},

	/// Get value at current key.
	#[command(name = "get")]
	Get {
		/// Key path to get. Supports relative key paths.
		key: Option<String>,

		/// Optional type hint for value parsing.
		#[arg(short = 't', long = "type")]
		type_hint: Option<String>,
	},

	/// List all keys under the current key.
	#[command(name = "ls")]
	List {
		/// Key path to list. Supports relative key paths.
		key: Option<String>,

		/// Max depth of keys shown.
		#[arg(short = 'd', long, default_value_t = 1)]
		max_depth: usize,
		/// Whether or not to hide subspace keys which are past the max depth.
		#[arg(short = 'u', long, default_value_t = false)]
		hide_subspaces: bool,
		/// Print style
		#[arg(short = 's', long, default_value = "tree")]
		style: ListStyle,
	},

	/// Set value at current path.
	#[command(name = "set")]
	Set {
		/// Key path to set. Supports relative key paths. Used as value if value is not set.
		key_or_value: String,

		/// Value to set, with optional type prefix (e.g. "u64:123", overrides type hint).
		value: Option<String>,
		/// Optional type hint.
		#[arg(short = 't', long)]
		type_hint: Option<String>,
	},

	/// Clear value or range at current path.
	#[command(name = "clear")]
	Clear {
		/// Key path to clear. Supports relative key paths.
		key: Option<String>,

		/// Clears the entire subspace range instead of just the key.
		#[arg(short = 'r', long = "range", default_value_t = false)]
		clear_range: bool,
		/// Disable confirmation prompt.
		#[arg(short = 'y', long, default_value_t = false)]
		yes: bool,
	},

	#[command(name = "exit")]
	Exit,
}

impl SubCommand {
	pub async fn execute(self, pool: &FdbPool, current_tuple: &mut SimpleTuple) -> CommandResult {
		match self {
			SubCommand::ChangeKey { key } => {
				update_current_tuple(current_tuple, Some(key));
			}
			SubCommand::Get { key, type_hint } => {
				let mut current_tuple = current_tuple.clone();
				if !update_current_tuple(&mut current_tuple, key) {
					return CommandResult::Error;
				}

				let fut = pool.run(|tx, _mc| {
					let current_tuple = current_tuple.clone();
					async move {
						let key = fdb::tuple::pack(&current_tuple);
						let entry = tx.get(&key, true).await?;
						Ok(entry)
					}
				});

				match tokio::time::timeout(Duration::from_secs(5), fut).await {
					Ok(Ok(entry)) => {
						if let Some(entry) = entry {
							match SimpleValue::parse_bytes(type_hint.as_deref(), &entry) {
								Ok(parsed) => println!("{parsed}"),
								Err(err) => println!("{err:#}"),
							}
						} else {
							println!("key does not exist");
						};
					}
					Ok(Err(err)) => println!("txn error: {err:#}"),
					Err(_) => println!("txn timed out"),
				}
			}
			SubCommand::List {
				key,
				max_depth,
				hide_subspaces,
				style: list_style,
			} => {
				let mut current_tuple = current_tuple.clone();
				if !update_current_tuple(&mut current_tuple, key) {
					return CommandResult::Error;
				}

				let subspace = fdb::tuple::Subspace::all().subspace(&current_tuple);

				let fut = pool.run(|tx, _mc| {
					let subspace = subspace.clone();
					async move {
						let entries = tx
							.get_ranges_keyvalues(
								fdb::RangeOption {
									mode: StreamingMode::WantAll,
									..(&subspace).into()
								},
								true,
							)
							.try_collect::<Vec<_>>()
							.await?;

						Ok(entries)
					}
				});

				match tokio::time::timeout(Duration::from_secs(5), fut).await {
					Ok(Ok(entries)) => {
						let mut entry_count = 0;
						let mut subspace_count = 0;
						let mut current_hidden_subspace: Option<SimpleTuple> = None;
						let mut hidden_count = 0;
						let mut last_key = SimpleTuple::new();

						for entry in &entries {
							match subspace.unpack::<SimpleTuple>(entry.key()) {
								Ok(key) => {
									if key.segments.len() <= max_depth {
										if entry.value().is_empty() {
											key.print(&list_style, &last_key);
											println!();
										} else {
											match SimpleValue::parse_bytes(None, entry.value()) {
												Ok(value) => {
													key.print(&list_style, &last_key);

													let indent = match list_style {
														ListStyle::List => "  ".to_string(),
														ListStyle::Tree => format!(
															"  {}",
															"| ".repeat(key.segments.len())
														),
													};
													println!(
														" = {}",
														indent_string(
															&value.to_string(),
															style(indent).dim().to_string(),
															true
														)
													);
												}
												Err(err) => {
													key.print(&list_style, &last_key);
													println!(": {err:#}");
												}
											}
										}

										last_key = key;

										if let Some(curr) = &current_hidden_subspace {
											curr.print(&list_style, &last_key);
											println!(
												"/ {}",
												style(format!(
													"{hidden_count} {}",
													if hidden_count == 1 {
														"entry"
													} else {
														"entries"
													}
												))
												.dim()
											);

											last_key = curr.clone();
											current_hidden_subspace = None;
											hidden_count = 0;
										}

										entry_count += 1;
									} else if !hide_subspaces {
										let sliced = key.slice(max_depth);

										if let Some(curr) = &current_hidden_subspace {
											if &sliced == curr {
												hidden_count += 1;
											} else {
												curr.print(&list_style, &last_key);
												println!(
													"/ {}",
													style(format!(
														"{hidden_count} {}",
														if hidden_count == 1 {
															"entry"
														} else {
															"entries"
														}
													))
													.dim()
												);

												last_key = curr.clone();
												current_hidden_subspace = Some(sliced);
												hidden_count = 1;
												subspace_count += 1;
											}
										} else {
											current_hidden_subspace = Some(sliced);
											hidden_count = 1;
											subspace_count += 1;
										}
									}
								}
								Err(err) => println!("error parsing key: {err:#}"),
							}
						}

						if !hide_subspaces {
							if let Some(curr) = current_hidden_subspace {
								curr.print(&list_style, &last_key);
								println!(
									"/ {}",
									style(format!(
										"{hidden_count} {}",
										if hidden_count == 1 {
											"entry"
										} else {
											"entries"
										}
									))
									.dim()
								);
							}
						}

						if !entries.is_empty() {
							println!();
						}

						print!(
							"{} {}",
							entry_count,
							if entry_count == 1 { "entry" } else { "entries" }
						);

						if subspace_count != 0 {
							print!(
								", {} {} ({} total entries)",
								subspace_count,
								if subspace_count == 1 {
									"subspace"
								} else {
									"subspaces"
								},
								entries.len()
							);
						}

						println!();
					}
					Ok(Err(err)) => println!("txn error: {err:#}"),
					Err(_) => println!("txn timed out"),
				}
			}
			SubCommand::Set {
				key_or_value,
				value,
				type_hint,
			} => {
				let (key, value) = if let Some(value) = value {
					(Some(key_or_value), value)
				} else {
					(None, key_or_value)
				};

				let mut current_tuple = current_tuple.clone();
				if !update_current_tuple(&mut current_tuple, key) {
					return CommandResult::Error;
				}

				let parsed_value = match SimpleValue::parse_str(type_hint.as_deref(), &value) {
					Ok(parsed) => parsed,
					Err(err) => {
						println!("{err:#}");
						return CommandResult::Error;
					}
				};

				let fut = pool.run(|tx, _mc| {
					let current_tuple = current_tuple.clone();
					let value = parsed_value.clone();
					async move {
						let key = fdb::tuple::pack(&current_tuple);
						let value = value
							.serialize()
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

						tx.set(&key, &value);
						Ok(())
					}
				});

				match tokio::time::timeout(Duration::from_secs(5), fut).await {
					Ok(Ok(_)) => {}
					Ok(Err(err)) => println!("txn error: {err:#}"),
					Err(_) => println!("txn timed out"),
				}
			}
			SubCommand::Clear {
				key,
				clear_range,
				yes,
			} => {
				let mut current_tuple = current_tuple.clone();
				if !update_current_tuple(&mut current_tuple, key) {
					return CommandResult::Error;
				}

				if !yes {
					let term = rivet_term::terminal();
					let response = rivet_term::prompt::PromptBuilder::default()
						.message("Are you sure?")
						.build()
						.expect("failed to build prompt")
						.bool(&term)
						.await
						.expect("failed to show prompt");
					if !response {
						return CommandResult::Error;
					}
				}

				let fut = pool.run(|tx, _mc| {
					let current_tuple = current_tuple.clone();
					async move {
						if clear_range {
							let subspace = fdb::tuple::Subspace::all().subspace(&current_tuple);
							tx.clear_subspace_range(&subspace);
						} else {
							let key = fdb::tuple::pack(&current_tuple);
							tx.clear(&key);
						}

						Ok(())
					}
				});

				match tokio::time::timeout(Duration::from_secs(5), fut).await {
					Ok(Ok(_)) => {}
					Ok(Err(err)) => println!("txn error: {err:#}"),
					Err(_) => println!("txn timed out"),
				}
			}
			SubCommand::Exit => return CommandResult::Exit,
		}

		CommandResult::Ok
	}
}

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq)]
pub enum ClearType {
	Range,
}

pub enum CommandResult {
	Ok,
	Error,
	Exit,
}

fn update_current_tuple(current_tuple: &mut SimpleTuple, key: Option<String>) -> bool {
	let Some(key) = key.as_deref() else {
		return true;
	};

	match SimpleTuple::parse(key) {
		Ok((parsed, relative, back_count)) => {
			if relative {
				for _ in 0..back_count {
					current_tuple.segments.pop();
				}
				current_tuple.segments.extend(parsed.segments);
			} else {
				*current_tuple = parsed;
			}

			true
		}
		Err(err) => {
			println!("{err:#}");

			false
		}
	}
}
