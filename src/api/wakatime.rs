use axum::{extract::State, Json};
use crate::services::wakatime::WakatimeData;
use crate::state::AppState;

pub async fn get_wakatime_stats(State(state): State<AppState>) -> Json<Option<WakatimeData>> {
    let data = state.wakatime_data.read().unwrap();
    Json(data.clone())
}
