//! Esportazione dei report: per ora CSV e JSON. Il PDF arriva nella tappa T6.
//!
//! Il CSV mantiene la convenzione del programma Python: separatore `;` e BOM iniziale (così Excel
//! italiano apre il file con le colonne giuste).

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use postcritica_archive::AnalysisRecord;

/// Esporta i record in CSV.
pub fn to_csv(records: &[&AnalysisRecord]) -> String {
    let mut out = String::from("\u{feff}"); // BOM
    out.push_str("id;nome;fonte;punteggio;tag\n");
    for record in records {
        out.push_str(&record.id.to_string());
        out.push(';');
        out.push_str(&escape_csv(&record.file_name));
        out.push(';');
        out.push_str(&escape_csv(&record.source_key));
        out.push(';');
        out.push_str(&record.score.map(|s| s.to_string()).unwrap_or_default());
        out.push(';');
        out.push_str(&escape_csv(&record.tags.join(", ")));
        out.push('\n');
    }
    out
}

/// Esporta un record in JSON. Scritto a mano, senza dipendenze, con l'escape minimo necessario.
pub fn record_to_json(record: &AnalysisRecord) -> String {
    let tags = record
        .tags
        .iter()
        .map(|t| format!("\"{}\"", escape_json(t)))
        .collect::<Vec<_>>()
        .join(",");
    let score = record
        .score
        .map(|s| s.to_string())
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{\"id\":{},\"nome\":\"{}\",\"fonte\":\"{}\",\"punteggio\":{},\"tag\":[{}]}}",
        record.id,
        escape_json(&record.file_name),
        escape_json(&record.source_key),
        score,
        tags,
    )
}

/// Esporta tutti i record in un array JSON.
pub fn to_json(records: &[&AnalysisRecord]) -> String {
    let items: Vec<String> = records.iter().map(|record| record_to_json(record)).collect();
    format!("[{}]", items.join(","))
}

/// Esporta i record in JSONL: un oggetto JSON per riga.
pub fn to_jsonl(records: &[&AnalysisRecord]) -> String {
    let mut out = String::new();
    for record in records {
        out.push_str(&record_to_json(record));
        out.push('\n');
    }
    out
}

/// Racchiude fra virgolette un campo che contiene il separatore, virgolette o a capo.
fn escape_csv(field: &str) -> String {
    if field.contains(';') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// L'escape minimo per una stringa JSON.
fn escape_json(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    for ch in field.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

/// Costruisce un PDF di una pagina con titolo e righe di testo.
///
/// Scritto a mano (niente dipendenze): oggetti, stream e tabella `xref` con gli offset di byte.
/// Il testo usa la codifica WinAnsi, così gli accenti italiani sopravvivono.
pub fn report_pdf(title: &str, lines: &[String]) -> Vec<u8> {
    let mut stream: Vec<u8> = Vec::new();
    stream.extend_from_slice(b"BT /F1 16 Tf 50 800 Td (");
    push_pdf_text(&mut stream, title);
    stream.extend_from_slice(b") Tj ET\n");
    let mut y = 775.0_f32;
    for line in lines {
        if y < 40.0 {
            break;
        }
        stream.extend_from_slice(format!("BT /F1 11 Tf 50 {y:.0} Td (").as_bytes());
        push_pdf_text(&mut stream, line);
        stream.extend_from_slice(b") Tj ET\n");
        y -= 15.0;
    }

    let objects: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
        {
            let mut object = format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes();
            object.extend_from_slice(&stream);
            object.extend_from_slice(b"endstream");
            object
        },
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec(),
    ];

    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref_offset = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// Scrive il testo nel PDF con l'escape e la codifica WinAnsi (Latin-1).
fn push_pdf_text(out: &mut Vec<u8>, text: &str) {
    for ch in text.chars() {
        match ch {
            '(' => out.extend_from_slice(b"\\("),
            ')' => out.extend_from_slice(b"\\)"),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\n' | '\r' => out.push(b' '),
            c if (c as u32) < 256 => out.push(c as u8),
            _ => out.push(b'?'),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> AnalysisRecord {
        let mut record = AnalysisRecord::new("vacanza;2024.jpg", "/foto/vacanza.jpg");
        record.id = 3;
        record.score = Some(88);
        record.tags = vec!["mare".to_string(), "tramonto".to_string()];
        record
    }

    #[test]
    fn csv_starts_with_a_bom_and_escapes_the_separator() {
        let r = record();
        let csv = to_csv(&[&r]);
        assert!(csv.starts_with('\u{feff}'));
        assert!(csv.contains("\"vacanza;2024.jpg\""));
        assert!(csv.contains("3;"));
    }

    #[test]
    fn json_has_the_expected_fields() {
        let json = record_to_json(&record());
        assert!(json.contains("\"punteggio\":88"));
        assert!(json.contains("\"mare\""));
        assert!(json.contains("\"tramonto\""));
    }

    #[test]
    fn a_pdf_has_a_header_a_body_and_an_end() {
        let pdf = report_pdf(
            "Prova",
            &["Punteggio: 80".to_string(), "Tag: mare (città)".to_string()],
        );
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.starts_with("%PDF-1.4"));
        assert!(text.contains("(Prova)"));
        assert!(text.contains("Punteggio: 80"));
        assert!(text.contains("xref"));
        assert!(text.trim_end().ends_with("%%EOF"));
    }

    #[test]
    fn a_pdf_escapes_parentheses() {
        let pdf = report_pdf("a (b) c", &[]);
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("(a \\(b\\) c)"));
    }

    #[test]
    fn a_missing_score_is_json_null() {
        let mut r = record();
        r.score = None;
        assert!(record_to_json(&r).contains("\"punteggio\":null"));
    }
}
