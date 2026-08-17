//! Stable Diffusion 图像生成(ComfyUI API)。
//!
//! 配置 `comfyui_url`(默认 `http://127.0.0.1:8188`)后,通过 ComfyUI
//! 的 `/prompt` 与 `/history` 接口提交内置 txt2img 工作流并取回图片,
//! 落盘 `data/sd/`。ComfyUI 未运行时报错清晰,不影响其它功能。

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

/// 生成结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdResult {
    pub status: String,
    pub filename: String,
    pub path: String,
    pub prompt_id: String,
}

/// 生成图片(同步,最多等待 120 秒)。
pub fn generate(state: &AppState, prompt: &str, negative: Option<&str>) -> Result<SdResult, String> {
    let base = state.comfyui_url.trim_end_matches('/').to_string();
    if base.is_empty() {
        return Err("未配置 ComfyUI 地址(config.toml 的 comfyui_url)".into());
    }

    // 内置 txt2img 工作流。
    let workflow = json!({
        "3": { "class_type": "KSampler", "inputs": {
            "seed": crate::state::now_secs() % 1000000000,
            "steps": 24, "cfg": 7.0, "sampler_name": "euler", "scheduler": "normal", "denoise": 1.0,
            "model": ["4", 0], "positive": ["6", 0], "negative": ["7", 0], "latent_image": ["5", 0],
        }},
        "4": { "class_type": "CheckpointLoaderSimple", "inputs": { "ckpt_name": "v1-5-pruned-emaonly.safetensors" }},
        "5": { "class_type": "EmptyLatentImage", "inputs": { "width": 512, "height": 512, "batch_size": 1 }},
        "6": { "class_type": "CLIPTextEncode", "inputs": { "text": prompt, "clip": ["4", 1] }},
        "7": { "class_type": "CLIPTextEncode", "inputs": { "text": negative.unwrap_or(""), "clip": ["4", 1] }},
        "8": { "class_type": "VAEDecode", "inputs": { "samples": ["3", 0], "vae": ["4", 2] }},
        "9": { "class_type": "SaveImage", "inputs": { "filename_prefix": "pilhome", "images": ["8", 0] }},
    });

    // 提交。
    let client_id = format!("pilhome-{}", crate::state::now_secs());
    let resp = ureq::post(&format!("{base}/prompt"))
        .timeout(Duration::from_secs(30))
        .send_json(json!({ "prompt": workflow, "client_id": client_id }))
        .map_err(|e| format!("ComfyUI 提交失败(服务未启动?):{e}"))?;
    let body: Value = resp.into_json().map_err(|e| format!("ComfyUI 响应解析失败:{e}"))?;
    let prompt_id = body["prompt_id"].as_str().ok_or(format!("ComfyUI 未返回 prompt_id:{body}"))?.to_string();

    // 轮询结果。
    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    loop {
        if std::time::Instant::now() > deadline {
            return Err("ComfyUI 生成超时(120s)".into());
        }
        std::thread::sleep(Duration::from_secs(2));
        let resp = ureq::get(&format!("{base}/history/{prompt_id}"))
            .timeout(Duration::from_secs(15))
            .call()
            .map_err(|e| format!("ComfyUI 查询失败:{e}"))?;
        let body: Value = resp.into_json().map_err(|e| format!("解析失败:{e}"))?;
        let Some(outputs) = body.get(&prompt_id).and_then(|v| v.get("outputs")) else { continue };
        let mut found: Option<(String, String)> = None; // (filename, subfolder)
        for (_, node_out) in outputs.as_object().unwrap_or(&serde_json::Map::new()) {
            if let Some(images) = node_out.get("images").and_then(|v| v.as_array()) {
                for img in images {
                    if let Some(fname) = img.get("filename").and_then(|v| v.as_str()) {
                        let sub = img.get("subfolder").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        found = Some((fname.to_string(), sub));
                    }
                }
            }
        }
        if let Some((filename, subfolder)) = found {
            // 下载图片。
            let url = format!("{base}/view?filename={filename}&subfolder={subfolder}");
            let resp = ureq::get(&url).timeout(Duration::from_secs(30)).call().map_err(|e| format!("下载图片失败:{e}"))?;
            let dir = state.data_dir.join("sd");
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let path = dir.join(&filename);
            let mut reader = resp.into_reader();
            let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
            std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
            return Ok(SdResult {
                status: "done".into(),
                filename,
                path: path.to_string_lossy().to_string(),
                prompt_id,
            });
        }
    }
}