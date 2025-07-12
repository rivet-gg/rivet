use anyhow::*;
use foundationdb::tuple::Subspace;
use indexmap::IndexMap;
use pegboard_config::runner_protocol::proto::kv;

use crate::{
	entry::EntryBuilder,
	key::{Key, ListKey},
	MAX_KEY_SIZE,
};

#[derive(Clone, Debug)]
pub enum ListQuery {
	All,
	Range {
		start: ListKey,
		end: Key,
		exclusive: bool,
	},
	Prefix(ListKey),
}

impl ListQuery {
	pub(crate) fn range(&self, subspace: &Subspace) -> (Vec<u8>, Vec<u8>) {
		match self {
			ListQuery::All => subspace.range(),
			ListQuery::Range {
				start,
				end,
				exclusive,
			} => (
				subspace.subspace(&start).range().0,
				if *exclusive {
					subspace.subspace(&end).range().0
				} else {
					subspace.subspace(&end).range().1
				},
			),
			ListQuery::Prefix(prefix) => subspace.subspace(&prefix).range(),
		}
	}

	pub(crate) fn validate(&self) -> Result<()> {
		match self {
			ListQuery::All => {}
			ListQuery::Range { start, end, .. } => {
				ensure!(
					start.len() <= MAX_KEY_SIZE,
					"start key is too long (max 2048 bytes)"
				);
				ensure!(
					end.len() <= MAX_KEY_SIZE,
					"end key is too long (max 2048 bytes)"
				);
			}
			ListQuery::Prefix(prefix) => {
				ensure!(
					prefix.len() <= MAX_KEY_SIZE,
					"prefix key is too long (max 2048 bytes)"
				);
			}
		}

		Ok(())
	}
}

impl TryFrom<kv::ListQuery> for ListQuery {
	type Error = Error;

	fn try_from(value: kv::ListQuery) -> Result<ListQuery> {
		match value.kind.context("ListQuery.kind")? {
			kv::list_query::Kind::All(_) => Ok(ListQuery::All),
			kv::list_query::Kind::Range(range) => Ok(ListQuery::Range {
				start: range.start.context("Range.start")?.into(),
				end: range.end.context("Range.end")?.into(),
				exclusive: range.exclusive,
			}),
			kv::list_query::Kind::Prefix(prefix) => {
				Ok(ListQuery::Prefix(prefix.key.context("Prefix.key")?.into()))
			}
		}
	}
}

impl From<ListQuery> for kv::ListQuery {
	fn from(value: ListQuery) -> kv::ListQuery {
		match value {
			ListQuery::All => kv::ListQuery {
				kind: Some(kv::list_query::Kind::All(kv::list_query::All {})),
			},
			ListQuery::Range {
				start,
				end,
				exclusive,
			} => kv::ListQuery {
				kind: Some(kv::list_query::Kind::Range(kv::list_query::Range {
					start: Some(start.into()),
					end: Some(end.into()),
					exclusive,
				})),
			},
			ListQuery::Prefix(key) => kv::ListQuery {
				kind: Some(kv::list_query::Kind::Prefix(kv::list_query::Prefix {
					key: Some(key.into()),
				})),
			},
		}
	}
}

// Used to short circuit after the
pub struct ListLimitReached(pub IndexMap<Key, EntryBuilder>);

impl std::fmt::Debug for ListLimitReached {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "ListLimitReached")
	}
}

impl std::fmt::Display for ListLimitReached {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "List limit reached")
	}
}

impl std::error::Error for ListLimitReached {}
