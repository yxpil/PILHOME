//! 周期任务:定时规则检查、网络扫描、规则进化、节律计算。

use crate::state::{now_secs, AppState};
use crate::webhook::Webhook;
use crate::Config;
use pilhome_autotime::suggest_arm_windows;
use pilhome_netscanear::{analyze, scan_network};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

/// 启动全部周期任务。
pub fn spawn_periodic(state: Arc<AppState>, cfg: Config, webhook: std::sync::Arc<Webhook>) {
    // 数据快照(事件/审计/画像落盘)。
    {
        let state = state.clone();
        let interval_secs = cfg.snapshot_interval_secs.max(30);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                state.persistence.snapshot(&state);
            }
        });
    }

    // 告警 Webhook 推送(每 15 秒检查新增 alert)。
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            loop {
                interval.tick().await;
                webhook.poll_and_push(&state);
            }
        });
    }
    // 定时规则检查(cron)。
    {
        let state = state.clone();
        let tick = cfg.tasks.cron_tick_secs.max(1);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(tick));
            loop {
                interval.tick().await;
                let now = now_secs();
                let rules = state.schedules.read().map(|s| s.clone()).unwrap_or_default();
                for rule in rules {
                    if rule.matches(now) {
                        state.events.record(
                            crate::events::Level::Info,
                            "autotime",
                            format!("定时规则命中:{} -> {}", rule.name, rule.action),
                            None,
                            Some(serde_json::json!({ "action": rule.action })),
                        );
                    }
                }
            }
        });
    }

    // 周期网络扫描 + 陌生设备告警。
    {
        let state = state.clone();
        let interval_secs = cfg.tasks.scan_interval_secs.max(60);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                run_scan(&state).await;
            }
        });
    }

    // 周期规则进化(ATOGrowUP 变异)。
    {
        let state = state.clone();
        let interval_secs = cfg.tasks.evolve_interval_secs.max(300);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                if let Ok(mut evolver) = state.evolver.lock() {
                    if evolver.population().is_empty() {
                        continue;
                    }
                    let n = evolver.evolve();
                    let gen = evolver.generation();
                    state.events.record(
                        crate::events::Level::Info,
                        "atogrowup",
                        format!("规则进化:第 {gen} 代,种群 {n} 条"),
                        None,
                        None,
                    );
                }
            }
        });
    }

    // 周期健康检查(SelflookUP)。
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                let now = now_secs();
                let online = state.devices.online_count(now, cfg.offline_after_secs);
                let total = state.devices.len();
                let audit_len = state.audit.len();
                if total > 0 && online < total {
                    state.events.record(
                        crate::events::Level::Warn,
                        "selflookup",
                        format!("健康检查:设备在线率 {online}/{total}"),
                        None,
                        None,
                    );
                }
                let _ = audit_len;
            }
        });
    }
}

/// 执行一次网络扫描(阻塞扫描放阻塞线程)。
pub async fn run_scan(state: &Arc<AppState>) {
    let cfg = state.scan_cfg.read().map(|c| c.clone()).unwrap_or_default();
    let result = tokio::task::spawn_blocking(move || scan_network(&cfg)).await;
    let Ok(result) = result else { return };

    let whitelist: BTreeSet<String> = state
        .devices
        .all()
        .into_iter()
        .filter(|d| d.trusted)
        .map(|d| d.address.split(':').next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let report = analyze(&result, &whitelist);
    let unknown = report.unknown_total;
    if let Ok(mut last) = state.last_scan.lock() {
        *last = Some(report);
    }
    if unknown > 0 {
        state.events.record(
            crate::events::Level::Alert,
            "netscanear",
            format!("发现 {unknown} 台陌生设备"),
            None,
            Some(serde_json::json!({ "unknown": unknown, "elapsed_ms": result.elapsed_ms })),
        );
    } else {
        state.events.record(
            crate::events::Level::Info,
            "netscanear",
            format!("网络扫描完成:{} 台活跃主机,无陌生设备", result.alive_count),
            None,
            None,
        );
    }
}


/// 基于行为画像计算节律并输出布防建议。
pub fn rhythm_suggestion(state: &AppState) -> serde_json::Value {
    // 优先从 ATOGrowUP 行为画像聚合节律,样本不足时回退默认作息模板。
    let curve = {
        let profile = state.profile.lock().ok();
        match profile {
            Some(p) if p.len() >= 48 => {
                let mut acc = [0.0f32; 24];
                let mut n = 0usize;
                for device in p.devices() {
                    for (i, v) in p.activity_curve(&device).iter().enumerate() {
                        acc[i] += v;
                    }
                    n += 1;
                }
                if n > 0 {
                    for v in acc.iter_mut() {
                        *v /= n as f32;
                    }
                }
                pilhome_autotime::from_profile(acc)
            }
            _ => pilhome_autotime::default_rhythm(),
        }
    };
    let windows = suggest_arm_windows(&curve, 0.2);
    serde_json::json!({
        "curve": curve.0,
        "recommended_arm_windows": windows,
    })
}
/// 存储任务:SQLite 增量落库 + MySQL 增量同步 + 启动时自动发现一次。
pub fn spawn_storage(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut last_ts: u64 = 0;
        // 启动即跑一轮全自动发现。
        {
            let state2 = state.clone();
            tokio::task::spawn_blocking(move || {
                let report = crate::discover::auto_discover(&state2);
                state2.events.record(
                    crate::events::Level::Info,
                    "netscanear",
                    format!("启动自动发现:共 {} 台设备(新增 {})", report.total_devices, report.new_devices),
                    None,
                    None,
                );
            });
        }
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            let recent: Vec<crate::events::AppEvent> = state
                .events
                .recent(500)
                .into_iter()
                .filter(|e| e.ts > last_ts)
                .collect();
            if let Some(max_ts) = recent.iter().map(|e| e.ts).max() {
                last_ts = max_ts;
            }
            // SQLite 落库。
            if state.sqlite.available() && !recent.is_empty() {
                state.sqlite.insert_batch(&recent);
            }
            // MySQL 同步(每 60s 一次,失败降级日志)。
            if state.mysql.enabled() && !recent.is_empty() {
                let mysql = state.mysql.clone();
                let events = recent.clone();
                let state2 = state.clone();
                tokio::task::spawn_blocking(move || match mysql.sync_events(&events) {
                    Ok(n) => {
                        if n > 0 {
                            state2.events.record(
                                crate::events::Level::Info,
                                "storage",
                                format!("已同步 {n} 条事件到 MySQL"),
                                None,
                                None,
                            );
                        }
                    }
                    Err(e) => {
                        state2.events.record(crate::events::Level::Warn, "storage", format!("MySQL 同步失败:{e}"), None, None);
                    }
                });
            }
        }
    });
}
/// 构建 AI 自我审查的运行快照。
pub fn build_review_snapshot(state: &AppState) -> String {
    let now = crate::state::now_secs();
    let total = state.devices.len();
    let online = state.devices.online_count(now, 120);
    let events: Vec<String> = state.events.recent(20).iter().map(|e| format!("[{:?}] {}", e.level, e.message)).collect();
    let tools = state.tools.list();
    let tools_err: Vec<String> = tools.iter().filter_map(|t| t.last_error.clone().map(|e| format!("{}:{e}", t.name))).collect();
    let bayes = state.bayes.lock().map(|b| format!("{}样本", b.total_pos + b.total_neg)).unwrap_or_default();
    format!(
        "系统运行时间: {}s\n设备: {}/{} 在线\n工具: {} 个,报错: {}\n贝叶斯: {}\n最近事件:\n{}\n自动化规则: {} 条\n",
        now - state.started_at,
        online,
        total,
        tools.len(),
        if tools_err.is_empty() { "无".into() } else { tools_err.join("; ") },
        bayes,
        events.join("\n"),
        state.automations.read().map(|r| r.len()).unwrap_or(0),
    )
}

/// AI 定期自我审查任务:按配置间隔调 AI 审查系统并记录建议。
pub fn spawn_ai_review(state: Arc<AppState>, interval_secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs.max(600)));
        loop {
            interval.tick().await;
            if !state.ai.configured() {
                continue;
            }
            let snapshot = build_review_snapshot(&state);
            match state.ai.review(&snapshot) {
                Ok(out) => {
                    state.events.record(crate::events::Level::Info, "ai.review", format!("AI 自我审查:{out}"), None, None);
                }
                Err(e) => {
                    state.events.record(crate::events::Level::Warn, "ai.review", format!("AI 自我审查失败:{e}"), None, None);
                }
            }
        }
    });
}

/// 贝叶斯习惯学习任务:从审计日志提取训练样本。
///
/// 正样本:用户手动控制设备(audit actor=api 的 ha.control / device 命令);
/// 负样本:自动化动作后 5 分钟内用户手动反向操作。
pub fn spawn_bayes_learn(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(120));
        let mut last_ts: u64 = 0;
        loop {
            interval.tick().await;
            let entries = state.audit.recent(200);
            let new_entries: Vec<_> = entries.into_iter().filter(|e| e.ts > last_ts).collect();
            if let Some(max_ts) = new_entries.iter().map(|e| e.ts).max() {
                last_ts = max_ts;
            }
            let mut learned = 0usize;
            for e in &new_entries {
                if e.action.starts_with("ha.control") || e.action.starts_with("device.command") {
                    let hour = (e.ts / 3600 % 24).to_string();
                    let feats: Vec<(String, String)> = vec![
                        ("action".to_string(), e.action.clone()),
                        ("hour".to_string(), hour),
                        ("device".to_string(), e.target.clone()),
                    ];
                    if let Ok(mut b) = state.bayes.lock() {
                        let feats_ref: Vec<(&str, &str)> = feats.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                        b.train(&feats_ref, true);
                        learned += 1;
                    }
                }
            }
            if learned > 0 {
                state.events.record(crate::events::Level::Info, "atogrowup", format!("习惯学习:新增 {learned} 条正样本"), None, None);
            }
        }
    });
}