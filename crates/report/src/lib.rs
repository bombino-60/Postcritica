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
    fn a_missing_score_is_json_null() {
        let mut r = record();
        r.score = None;
        assert!(record_to_json(&r).contains("\"punteggio\":null"));
    }
}
