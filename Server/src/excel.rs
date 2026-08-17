//! Excel / CSV 导出。
//!
//! `export_xlsx` 生成标准 .xlsx(OOXML,zip 容器),Excel/WPS 可直接打开;
//! `export_csv` 生成 UTF-8 BOM CSV(Excel 双击即开)。

use crate::events::AppEvent;
use std::io::Write;
use std::path::Path;

/// 导出事件为 .xlsx 文件。
pub fn export_xlsx(events: &[AppEvent], path: &Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("创建文件失败:{e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1) [Content_Types].xml
    zip.start_file("[Content_Types].xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#.as_bytes(),
    )
    .map_err(|e| e.to_string())?;

    // 2) _rels/.rels
    zip.start_file("_rels/.rels", opts).map_err(|e| e.to_string())?;
    zip.write_all(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#.as_bytes(),
    )
    .map_err(|e| e.to_string())?;

    // 3) xl/workbook.xml
    zip.start_file("xl/workbook.xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="事件" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#.as_bytes(),
    )
    .map_err(|e| e.to_string())?;

    // 4) xl/_rels/workbook.xml.rels
    zip.start_file("xl/_rels/workbook.xml.rels", opts).map_err(|e| e.to_string())?;
    zip.write_all(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#.as_bytes(),
    )
    .map_err(|e| e.to_string())?;

    // 5) xl/worksheets/sheet1.xml
    zip.start_file("xl/worksheets/sheet1.xml", opts).map_err(|e| e.to_string())?;
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
    );
    // 表头。
    xml.push_str("<row r=\"1\">");
    for (col, title) in ["时间", "级别", "来源", "消息", "设备"].iter().enumerate() {
        xml.push_str(&format!(r#"<c r="{}{}" t="inlineStr"><is><t>{}</t></is></c>"#, col_name(col), 1, escape(title)));
    }
    xml.push_str("</row>");
    // 数据行。
    for (i, e) in events.iter().enumerate() {
        let row = i + 2;
        xml.push_str(&format!("<row r=\"{row}\">"));
        let vals = [
            e.ts.to_string(),
            format!("{:?}", e.level).to_lowercase(),
            e.source.clone(),
            e.message.clone(),
            e.device_id.clone().unwrap_or_default(),
        ];
        for (col, v) in vals.iter().enumerate() {
            xml.push_str(&format!(
                r#"<c r="{}{}" t="inlineStr"><is><t>{}</t></is></c>"#,
                col_name(col),
                row,
                escape(v)
            ));
        }
        xml.push_str("</row>");
    }
    xml.push_str("</sheetData></worksheet>");
    zip.write_all(xml.as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// 导出事件为 CSV(UTF-8 BOM)。
pub fn export_csv(events: &[AppEvent], path: &Path) -> Result<(), String> {
    let mut out = Vec::new();
    out.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // BOM,Excel 识别 UTF-8
    out.extend_from_slice("时间,级别,来源,消息,设备\n".as_bytes());
    for e in events {
        let line = format!(
            "{},{},{},{},{}\n",
            e.ts,
            format!("{:?}", e.level).to_lowercase(),
            csv_escape(&e.source),
            csv_escape(&e.message),
            csv_escape(e.device_id.as_deref().unwrap_or("")),
        );
        out.extend_from_slice(line.as_bytes());
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

/// XML 转义。
fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// CSV 转义(含逗号/引号时加引号)。
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// 列名(A、B、C…Z、AA…)。
fn col_name(i: usize) -> String {
    let mut n = i;
    let mut s = String::new();
    loop {
        s.insert(0, (b'A' + (n % 26) as u8) as char);
        n = n / 26;
        if n == 0 {
            break;
        }
        n -= 1;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Level;
    use serde_json::json;

    fn sample_events() -> Vec<AppEvent> {
        vec![
            AppEvent {
                ts: 1_785_900_000,
                level: Level::Alert,
                source: "netscanear".into(),
                message: "发现 1 台陌生设备".into(),
                device_id: None,
                data: Some(json!({ "unknown": 1 })),
            },
            AppEvent {
                ts: 1_785_900_060,
                level: Level::Info,
                source: "mqtt".into(),
                message: "door:contact=open".into(),
                device_id: Some("door".into()),
                data: None,
            },
        ]
    }

    #[test]
    fn csv_export_has_bom_and_rows() {
        let path = std::env::temp_dir().join("pilhome_test.csv");
        export_csv(&sample_events(), &path).expect("csv export ok");
        let bytes = std::fs::read(&path).expect("read csv");
        assert_eq!(&bytes[0..3], &[0xEF, 0xBB, 0xBF]);
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("netscanear"));
        assert!(text.contains("door:contact=open"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn xlsx_export_is_valid_zip() {
        let path = std::env::temp_dir().join("pilhome_test.xlsx");
        export_xlsx(&sample_events(), &path).expect("xlsx export ok");
        let file = std::fs::File::open(&path).expect("open xlsx");
        let mut archive = zip::ZipArchive::new(file).expect("valid zip");
        assert!(archive.by_name("[Content_Types].xml").is_ok());
        assert!(archive.by_name("xl/workbook.xml").is_ok());
        let sheet = archive.by_name("xl/worksheets/sheet1.xml").expect("sheet");
        let mut content = String::new();
        use std::io::Read;
        let mut sheet = sheet;
        sheet.read_to_string(&mut content).expect("read sheet");
        assert!(content.contains("netscanear"));
        assert!(content.contains("发现 1 台陌生设备"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn column_names() {
        assert_eq!(col_name(0), "A");
        assert_eq!(col_name(25), "Z");
        assert_eq!(col_name(26), "AA");
    }
}