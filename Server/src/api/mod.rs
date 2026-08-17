//! REST API 路由汇总。

use crate::state::AppState;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub mod audit;
pub mod bayes;
pub mod ai;
pub mod automations;
pub mod control;
pub mod devices;
pub mod discover;
pub mod events;
pub mod export;
pub mod media;
pub mod evolve;
pub mod gesture;
pub mod ha;
pub mod health;
pub mod models;
pub mod sd;
pub mod mqtt;
pub mod profile;
pub mod scan;
pub mod schedule;
pub mod status;
pub mod token;
pub mod tools;
pub mod tts;
pub mod video;
pub mod wol;

/// 构建 /api 路由。
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/devices", get(devices::list).post(devices::upsert))
        .route("/api/devices/{id}/event", post(devices::event))
        .route("/api/devices/{id}/trust", post(devices::set_trust))
        .route("/api/events", get(events::list))
        .route("/api/scan/run", post(scan::run))
        .route("/api/scan/last", get(scan::last))
        .route("/api/status", get(status::overview))
        .route("/api/models", get(models::list).post(models::load))
        .route("/api/control", get(control::list))
        .route("/api/control/{id}/start", post(control::start))
        .route("/api/control/{id}/stop", post(control::stop))
        .route("/api/audit", get(audit::list))
        .route("/api/schedules", get(schedule::list).post(schedule::add))
        .route("/api/rhythm", get(schedule::rhythm))
        .route("/api/gesture", post(gesture::infer))
        .route("/api/profile", get(profile::view))
        .route("/api/evolve", get(evolve::view))
        .route("/api/evolve/seed", post(evolve::seed))
        .route("/api/evolve/tick", post(evolve::tick))
        .route("/api/video/probe", post(video::probe))
        .route("/api/video/audit", post(video::audit))
        .route("/api/health", get(health::report))
        .route("/api/ha/entities", get(ha::entities))
        .route("/api/ha/control", post(ha::control))
        .route("/api/ha/sync", post(ha::sync))
        .route("/api/ha/scenes", post(ha::scenes))
        .route("/api/automations", get(automations::list).post(automations::add))
        .route("/api/automations/{id}/toggle", post(automations::toggle))
        .route("/api/automations/{id}", axum::routing::delete(automations::remove))
        .route("/api/mqtt/publish", post(mqtt::publish))
        .route("/api/ws", get(crate::ws::ws_handler))
        .route("/api/token", post(token::issue))
        .route("/api/token/status", get(token::status))
        .route("/api/token/revoke", post(token::revoke))
        .route("/api/discover", post(discover::run))
        .route("/api/arp", get(discover::arp))
        .route("/api/vendors", get(discover::vendors))
        .route("/api/wol", post(wol::wake))
        .route("/api/export/events", get(export::events))
        .route("/api/mcp", axum::routing::post(crate::mcp::handler))
        .route("/api/ai/chat", post(ai::chat))
        .route("/api/ai/analyze", post(ai::analyze))
        .route("/api/ai/compress", post(ai::compress))
        .route("/api/ai/vision", post(ai::vision))
        .route("/api/ai/review", post(ai::review))
        .route("/api/ai/status", get(ai::status))
        .route("/api/tools", get(tools::list).post(tools::create))
        .route("/api/tools/search", get(tools::search))
        .route("/api/tools/{id}/run", post(tools::run))
        .route("/api/tools/{id}/debug", post(tools::debug))
        .route("/api/tools/{id}", axum::routing::delete(tools::remove))
        .route("/api/media/discover", get(media::discover))
        .route("/api/media/dial", post(media::dial))
        .route("/api/bayes/status", get(bayes::status))
        .route("/api/bayes/predict", post(bayes::predict))
        .route("/api/bayes/train", post(bayes::train))
        .route("/api/tts", post(tts::speak))
        .route("/api/sd/generate", post(sd::generate))
}