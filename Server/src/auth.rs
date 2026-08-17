//! API Token 鉴权:签发 / 校验 Bearer Token。
//!
//! 启用方式:`config.toml` 中 `auth_enabled = true`(默认关闭,不影响本机使用)。
//! 启用后,除 `/api/token`、`/api/ws` 与静态资源外,所有 `/api/*`
//! 请求需携带 `Authorization: Bearer <token>` 头。

use crate::state::AppState;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

/// 令牌库:并发安全的已签发令牌集合。
#[derive(Debug, Default)]
pub struct TokenStore {
    tokens: RwLock<HashSet<String>>,
}

impl TokenStore {
    /// 创建令牌库,预置配置中的令牌。
    pub fn new(preconfigured: &[String]) -> Self {
        let mut set = HashSet::new();
        for t in preconfigured {
            if !t.is_empty() {
                set.insert(t.clone());
            }
        }
        Self { tokens: RwLock::new(set) }
    }

    /// 签发新令牌(密码学随机 32 字节 → 64 位 hex)。
    pub fn issue(&self) -> String {
        let mut buf = [0u8; 32];
        let _ = getrandom::getrandom(&mut buf);
        let token: String = buf.iter().map(|b| format!("{b:02x}")).collect();
        self.tokens.write().expect("token store poisoned").insert(token.clone());
        token
    }

    /// 吊销令牌。
    pub fn revoke(&self, token: &str) -> bool {
        self.tokens.write().expect("token store poisoned").remove(token)
    }

    /// 校验令牌。
    pub fn verify(&self, token: &str) -> bool {
        self.tokens.read().expect("token store poisoned").contains(token)
    }

    /// 已签发数量。
    pub fn count(&self) -> usize {
        self.tokens.read().expect("token store poisoned").len()
    }
}

/// 鉴权中间件(在路由层按需挂载)。
pub async fn require_token(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 静态资源与 WebSocket 放行(WS 由网关内网约束)。
    let path = req.uri().path().to_string();
    if path == "/" || path.starts_with("/assets/") || path == "/api/token" || path == "/api/ws" {
        return Ok(next.run(req).await);
    }

    // 校验 Bearer 令牌。
    let header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = header.strip_prefix("Bearer ").unwrap_or("");
    if state.tokens.verify(token) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}