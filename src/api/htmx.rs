use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
    http::StatusCode,
};
use askama::Template;
use std::sync::Arc;
use crate::state::AppState;
use crate::domain::{LogEntry, LogQuery};
use crate::services::log_service;

// Wrapper struct for templates to implement IntoResponse
pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}

// Templates
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub total_events: usize,
    pub unique_ips: usize,
    pub active_users: u32,
    pub uptime: String,
    pub cpu: String,
    pub ram: String,
    
    pub logs: Vec<LogEntry>,
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
    pub q: String,
    pub sort_by: String,
    pub order: String,
}

#[derive(Template)]
#[template(path = "components/stats.html")]
pub struct StatsTemplate {
    pub total_events: usize,
    pub unique_ips: usize,
    pub active_users: u32,
    pub uptime: String,
    pub cpu: String,
    pub ram: String,
}

#[derive(Template)]
#[template(path = "components/table.html")]
pub struct TableTemplate {
    pub logs: Vec<LogEntry>,
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
    pub q: String,
    pub sort_by: String,
    pub order: String,
}

// Handlers

pub async fn dashboard_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let (logs, meta, stats) = log_service::fetch_logs(&params);
    let (uptime, cpu, ram) = get_system_metrics(&state);

    HtmlTemplate(DashboardTemplate {
        total_events: meta.total,
        unique_ips: stats.unique_ips,
        active_users: stats.active_users,
        uptime,
        cpu,
        ram,
        
        logs,
        page: meta.page,
        page_size: meta.page_size,
        total: meta.total,
        total_pages: meta.total_pages,
        q: params.q.unwrap_or_default(),
        sort_by: params.sort_by,
        order: params.order,
    })
}

pub async fn stats_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let params = LogQuery {
        page: 1,
        page_size: 1,
        q: None,
        sort_by: "timestamp".to_string(),
        order: "desc".to_string(),
    };
    
    let (_, meta, stats) = log_service::fetch_logs(&params);
    let (uptime, cpu, ram) = get_system_metrics(&state);

    HtmlTemplate(StatsTemplate {
        total_events: meta.total,
        unique_ips: stats.unique_ips,
        active_users: stats.active_users,
        uptime,
        cpu,
        ram,
    })
}

pub async fn logs_handler(
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let (logs, meta, _) = log_service::fetch_logs(&params);

    HtmlTemplate(TableTemplate {
        logs,
        page: meta.page,
        page_size: meta.page_size,
        total: meta.total,
        total_pages: meta.total_pages,
        q: params.q.unwrap_or_default(),
        sort_by: params.sort_by,
        order: params.order,
    })
}

// Helper for System Metrics
fn get_system_metrics(state: &Arc<AppState>) -> (String, String, String) {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let uptime_sec = state.start_time.elapsed().as_secs();
    let hrs = uptime_sec / 3600;
    let mins = (uptime_sec % 3600) / 60;
    let secs = uptime_sec % 60;
    let uptime = format!("{}h {}m {}s", hrs, mins, secs);

    let cpu = format!("{:.1}", sys.global_cpu_usage());
    let ram = format!("{}MB / {}MB", sys.used_memory() / 1024 / 1024, sys.total_memory() / 1024 / 1024);
    
    (uptime, cpu, ram)
}
