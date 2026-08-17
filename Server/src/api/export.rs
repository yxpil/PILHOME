//! 数据导出:Excel(.xlsx)/ CSV。

use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

/// 导出参数。
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    /// xlsx / csv(默认 csv)。
    pub format: Option<String>,
    /// 条数上限(默认 5000)。
    pub limit: Option<usize>,
}

/// GET /api/export/events?format=xlsx|csv —— 导出事件数据。
pub async fn events(State(state): State<Arc<AppState>>, Query(q): Query<ExportQuery>) -> Response {
    let limit = q.limit.unwrap_or(5000).min(50000);
    // 优先从 SQLite 读(数据全),否则内存事件。
    let events = if state.sqlite.available() {
        state.sqlite.recent(limit)
    } else {
        state.events.recent(limit)
    };

    let tmp = std::env::temp_dir().join(format!("pilhome_export_{}.{}", crate::state::now_secs(), q.format.as_deref().unwrap_or("csv")));
    let fmt = q.format.as_deref().unwrap_or("csv");
    let result = if fmt == "xlsx" {
        crate::excel::export_xlsx(&events, &tmp)
    } else {
        crate::excel::export_csv(&events, &tmp)
    };

    match result {
        Ok(()) => {
            let bytes = match std::fs::read(&tmp) {
                Ok(b) => b,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
            };
            let _ = std::fs::remove_file(&tmp);
            let ctype = if fmt == "xlsx" { "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" } else { "text/csv" };
            let filename = format!("pilhome_events.{}", fmt);
            let mut resp = ([(CONTENT_TYPE, ctype.to_string()), (CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\""))], bytes).into_response();
            resp.headers_mut().insert(CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\"").parse().unwrap());
            resp
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))).into_response(),
    }
}