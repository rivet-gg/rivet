use std::{
	future::Future,
	ops::{Deref, DerefMut},
};

use sqlx::sqlite::SqliteQueryResult;
use sqlx::{
	pool::PoolConnection, Executor, Sqlite, SqliteConnection, SqlitePool, TransactionManager,
};

pub trait SqlitePoolExt {
	fn conn(&self) -> impl Future<Output = sqlx::Result<PoolConnection<Sqlite>>>;
	fn begin_immediate<'a, 'b>(&'a self) -> impl Future<Output = sqlx::Result<Transaction<'b>>>;
}

impl SqlitePoolExt for SqlitePool {
	async fn conn(&self) -> sqlx::Result<PoolConnection<Sqlite>> {
		// Attempt to use an existing connection
		if let Some(conn) = self.try_acquire() {
			Ok(conn)
		} else {
			// Create a new connection
			self.acquire().await
		}
	}

	async fn begin_immediate<'a, 'b>(&'a self) -> sqlx::Result<Transaction<'b>> {
		let mut conn = self.conn().await?;

		conn.execute("BEGIN IMMEDIATE").await?;

		Ok(Transaction {
			conn: Conn::Owned(conn),
			is_open: true,
		})
	}
}

pub trait SqliteConnectionExt {
	fn begin_immediate(&mut self) -> impl Future<Output = sqlx::Result<Transaction>>;
}

impl SqliteConnectionExt for PoolConnection<Sqlite> {
	async fn begin_immediate(&mut self) -> sqlx::Result<Transaction> {
		let mut conn = Conn::Borrowed(self);

		conn.execute("BEGIN IMMEDIATE").await?;

		Ok(Transaction {
			conn,
			is_open: true,
		})
	}
}

enum Conn<'b> {
	Owned(PoolConnection<Sqlite>),
	Borrowed(&'b mut PoolConnection<Sqlite>),
}

impl<'c> Deref for Conn<'c> {
	type Target = PoolConnection<Sqlite>;

	fn deref(&self) -> &Self::Target {
		match self {
			Conn::Owned(conn) => &conn,
			Conn::Borrowed(conn) => conn,
		}
	}
}

impl<'c> DerefMut for Conn<'c> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Conn::Owned(conn) => conn,
			Conn::Borrowed(conn) => conn,
		}
	}
}

pub struct Transaction<'c> {
	conn: Conn<'c>,
	/// is the transaction open?
	is_open: bool,
}

impl<'c> Transaction<'c> {
	pub async fn commit(mut self) -> sqlx::Result<SqliteQueryResult> {
		let res = self.conn.execute("COMMIT").await;

		if res.is_ok() {
			self.is_open = false;
		}

		res
	}
}

impl<'c> Drop for Transaction<'c> {
	fn drop(&mut self) {
		if self.is_open {
			<Sqlite as sqlx::Database>::TransactionManager::start_rollback(&mut self.conn);
		}
	}
}

impl<'c> Deref for Transaction<'c> {
	type Target = SqliteConnection;

	fn deref(&self) -> &Self::Target {
		&*self.conn
	}
}

impl<'c> DerefMut for Transaction<'c> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.conn
	}
}
