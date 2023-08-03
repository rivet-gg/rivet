#[macro_export]
macro_rules! cass_prepared_statement {
	($name:ident => $expr:expr) => {
		pub mod $name {
			use super::*;

			pub(super) static PREPARED_STATEMENT_: tokio::sync::OnceCell<
				scylla::statement::prepared_statement::PreparedStatement,
			> = tokio::sync::OnceCell::const_new();

			pub async fn prepare(
				session: &scylla::transport::session::Session,
			) -> Result<
				&scylla::statement::prepared_statement::PreparedStatement,
				scylla::transport::errors::QueryError,
			> {
				$name::PREPARED_STATEMENT_
					.get_or_try_init(|| async {
						tracing::info!(statement = stringify!($name), "preparing statement");
						session.prepare($expr).await
					})
					.await
			}
		}
	};
}
