use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
    http::StatusCode,
};
use askama::Template;
use std::env;
use crate::state::AppState;
use crate::domain::{LogEntry, LogQuery, NavItem};
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
#[template(path = "dashboard.htmx", escape = "html")]
pub struct DashboardTemplate {
    pub username: String,
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
    
    pub nav_items: Vec<NavItem>,
}

#[derive(Template)]
#[template(path = "components/stats.htmx", escape = "html")]
pub struct StatsTemplate {
    pub total_events: usize,
    pub unique_ips: usize,
    pub active_users: u32,
    pub uptime: String,
    pub cpu: String,
    pub ram: String,
}

#[derive(Template)]
#[template(path = "components/table.htmx", escape = "html")]
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

#[derive(Template)]
#[template(path = "logged_out.htmx", escape = "html")]
pub struct LoggedOutTemplate;

// Handlers

pub async fn dashboard_handler(
    State(state): State<AppState>,
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let (logs, meta, stats) = log_service::fetch_logs(&params);
    let (uptime, cpu, ram) = get_system_metrics(&state);

    let username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());

    HtmlTemplate(DashboardTemplate {
        username,
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
        
        nav_items: get_nav_menu("/admin"),
    })
}

fn get_nav_menu(current_path: &str) -> Vec<NavItem> {
    vec![
        NavItem {
            label: "Overview".to_string(),
            href: "/admin".to_string(),
            active: current_path == "/admin",
        },
        // Future modules can be added here easily
    ]
}

pub async fn stats_handler(
    State(state): State<AppState>,
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

pub async fn logged_out_handler() -> impl IntoResponse {
    HtmlTemplate(LoggedOutTemplate)
}

// Helper for System Metrics
fn get_system_metrics(state: &AppState) -> (String, String, String) {
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
