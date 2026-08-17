//! TTS 语音合成。
//!
//! 支持两种方式:
//! 1. **本地 Edge-TTS 桥**:`config.toml` 配 `tts_url`(如 `http://127.0.0.1:5000/tts`),
//!    由第三方 Edge-TTS 服务(如 `edge-tts` CLI 包装)生成音频并保存;
//! 2. 输出到 `data/tts/` 目录,文件名可播报(`tts/<ts>.mp3`)。
//!
//! 未配置时返回明确错误,不影响网关其它功能。

use crate::state::AppState;

/// 合成语音:调用配置的 TTS 服务,音频落盘 `data/tts/`。
pub fn speak(state: &AppState, text: &str) -> Result<String, String> {
    let url = state.tts_url.clone();
    if url.is_empty() {
        return Err("未配置 TTS 服务(config.toml 的 tts_url)".into());
    }
    // 兼容两种形态:URL 含 {text} 占位则替换,否则作为 JSON POST 端点。
    let dir = state.data_dir.join("tts");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let out_path = dir.join(format!("{}.mp3", crate::state::now_secs()));
    let out = out_path.to_str().ok_or("路径错误")?.to_string();

    if url.contains("{text}") {
        let final_url = url.replace("{text}", &urlencoding(text));
        let resp = ureq::get(&final_url)
            .timeout(std::time::Duration::from_secs(60))
            .call()
            .map_err(|e| format!("TTS 请求失败:{e}"))?;
        let mut reader = resp.into_reader();
        let mut file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    } else {
        let resp = ureq::post(&url)
            .timeout(std::time::Duration::from_secs(60))
            .send_json(serde_json::json!({ "text": text }))
            .map_err(|e| format!("TTS 请求失败:{e}"))?;
        let mut reader = resp.into_reader();
        let mut file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// 简易 URL 编码(仅文本安全字符外转义)。
fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b' ' => {
                if b == b' ' { out.push('+'); } else { out.push(b as char); }
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}