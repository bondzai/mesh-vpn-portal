use crate::domain::{LogEntry, LogQuery, NavItem};
use crate::state::AppState;
use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::env;

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
    pub unique_device_ids: usize,
    pub active_users: u32,
    pub uptime: String,
    pub cpu: String,
    pub ram: String,
    pub nav_items: Vec<NavItem>,
    pub unique_ips: usize,
    pub top_ips: Vec<(String, u32)>,
    pub chart_labels: String,
    pub chart_data: String,
}

#[derive(Template)]
#[template(path = "components/overview.htmx", escape = "html")]
pub struct OverviewTemplate {
    pub active_users: u32,
    pub total_events: usize,
    pub unique_device_ids: usize,
    pub unique_ips: usize,
    pub uptime: String,
    pub cpu: String,
    pub ram: String,
    pub top_ips: Vec<(String, u32)>,
    pub chart_labels: String,
    pub chart_data: String,
}

#[derive(Template)]
#[template(path = "components/logs.htmx", escape = "html")]
pub struct LogsTemplate {
    pub q: String,
    pub exclude_ip: String,
    pub logs: Vec<LogEntry>,
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
    pub sort_by: String,
    pub order: String,
}

#[derive(Template)]
#[template(path = "components/stats.htmx", escape = "html")]
pub struct StatsTemplate {
    pub total_events: usize,
    pub unique_ips: usize,
    pub unique_device_ids: usize,
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
    pub exclude_ip: String,
    pub sort_by: String,
    pub order: String,
}

#[derive(Clone, Debug)]
pub struct ActiveUserDisplay {
    pub device_id: String,
    pub ip: String,
    pub device: String,
    pub duration: String,
}

#[derive(Template)]
#[template(path = "components/active_users.htmx", escape = "html")]
pub struct ActiveUsersTemplate {
    pub users: Vec<ActiveUserDisplay>,
}

#[derive(Template)]
#[template(path = "logged_out.htmx", escape = "html")]
pub struct LoggedOutTemplate;

// Handlers

pub async fn dashboard_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Only need stats for initial overview load
    let params = LogQuery {
        page: 1,
        page_size: 1,
        q: None,
        exclude_ip: None,
        sort_by: "timestamp".to_string(),
        order: "desc".to_string(),
    };

    let (_, meta, stats) = state.log_repository.find_all(&params);
    let (uptime, cpu, ram) = get_system_metrics(&state);

    let username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());

    // Prepare chart data
    let labels: Vec<String> = stats.requests_over_time.iter().map(|(t, _)| t.clone()).collect();
    let data: Vec<u32> = stats.requests_over_time.iter().map(|(_, c)| *c).collect();

    HtmlTemplate(DashboardTemplate {
        username,
        total_events: meta.total,
        unique_device_ids: stats.unique_device_ids,
        active_users: stats.active_users,
        uptime,
        cpu,
        ram,
        nav_items: get_nav_menu("/admin"),
        unique_ips: stats.unique_ips,
        top_ips: stats.top_ips,
        chart_labels: serde_json::to_string(&labels).unwrap_or_default(),
        chart_data: serde_json::to_string(&data).unwrap_or_default(),
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

pub async fn overview_tab_handler(State(state): State<AppState>) -> impl IntoResponse {
    let params = LogQuery {
        page: 1,
        page_size: 1,
        q: None,
        exclude_ip: None,
        sort_by: "timestamp".to_string(),
        order: "desc".to_string(),
    };

    let (_, meta, stats) = state.log_repository.find_all(&params);
    let (uptime, cpu, ram) = get_system_metrics(&state);

    // Prepare chart data
    let labels: Vec<String> = stats
        .requests_over_time
        .iter()
        .map(|(t, _)| t.clone())
        .collect();
    let data: Vec<u32> = stats.requests_over_time.iter().map(|(_, c)| *c).collect();

    HtmlTemplate(OverviewTemplate {
        active_users: stats.active_users,
        total_events: meta.total,
        unique_device_ids: stats.unique_device_ids,
        unique_ips: stats.unique_ips,
        uptime,
        cpu,
        ram,
        top_ips: stats.top_ips,
        chart_labels: serde_json::to_string(&labels).unwrap_or_default(),
        chart_data: serde_json::to_string(&data).unwrap_or_default(),
    })
}

pub async fn logs_tab_handler(
    State(state): State<AppState>,
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let (logs, meta, _) = state.log_repository.find_all(&params);

    HtmlTemplate(LogsTemplate {
        q: params.q.unwrap_or_default(),
        exclude_ip: params.exclude_ip.unwrap_or_default(),
        logs,
        page: meta.page,
        page_size: meta.page_size,
        total: meta.total,
        total_pages: meta.total_pages,
        sort_by: params.sort_by,
        order: params.order,
    })
}

pub async fn logs_handler(
    State(state): State<AppState>,
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let (logs, meta, _) = state.log_repository.find_all(&params);

    HtmlTemplate(TableTemplate {
        logs,
        page: meta.page,
        page_size: meta.page_size,
        total: meta.total,
        total_pages: meta.total_pages,
        q: params.q.unwrap_or_default(),
        exclude_ip: params.exclude_ip.unwrap_or_default(),
        sort_by: params.sort_by,
        order: params.order,
    })
}

pub async fn active_users_tab_handler(State(state): State<AppState>) -> impl IntoResponse {
    let connections = state.get_active_users();
    let users: Vec<ActiveUserDisplay> = connections
        .iter()
        .map(|c| {
            let duration = c.connected_at.elapsed();
            let secs = duration.as_secs();
            let duration_str = crate::utils::format_duration(secs);
            ActiveUserDisplay {
                device_id: c.device_id.clone(),
                ip: c.ip.clone(),
                device: c.device.clone(),
                duration: duration_str,
            }
        })
        .collect();

    HtmlTemplate(ActiveUsersTemplate { users })
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
    let ram = format!(
        "{}MB / {}MB",
        sys.used_memory() / 1024 / 1024,
        sys.total_memory() / 1024 / 1024
    );

    (uptime, cpu, ram)
}
