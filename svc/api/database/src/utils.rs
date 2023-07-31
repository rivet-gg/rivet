use api_helper::anchor::WatchResponse;
use rivet_api::models;

pub fn watch_response(value: WatchResponse) -> models::WatchResponse {
	models::WatchResponse {
		index: value.watch_index().to_owned(),
	}
}
