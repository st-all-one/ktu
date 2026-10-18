//! `check-adr` — valida os registos de governança: `wiki/adr/` e `wiki/postmortems/`.
//!
//! Contrato (ver `wiki/adr/README.md` e `wiki/postmortems/README.md`):
//! cada registo tem id único, título com o id, as **secções obrigatórias** (nas ADRs
//! `Alternatives considered` é a que impede re-litigar), e a substituição liga a ids
//! que existem (nos dois sentidos); o índice lista todos os registos.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Directório das ADRs.
const ADR_DIR: &str = "wiki/adr";
/// Directório dos postmortems.
const PM_DIR: &str = "wiki/postmortems";

/// Secções obrigatórias de uma ADR (a ordem não importa).
const ADR_SECTIONS: [&str; 4] = [
    "## Contexto",
    "## Decisão",
    "## Alternatives considered",
    "## Consequências",
];

/// Secções obrigatórias de um postmortem.
const PM_SECTIONS: [&str; 5] = [
    "## Contexto",
    "## Impacto",
    "## Causa",
    "## Correção",
    "## Prevenção",
];

/// Um registo lido (id, caminho e texto).
struct Record {
    /// Identificador (`0016` ou `PM-0002`).
    id: String,
    /// Caminho do ficheiro.
    path: PathBuf,
    /// Conteúdo integral.
    text: String,
}

/// Verifica as ADRs e os postmortems; devolve a lista de problemas (vazia = ok).
pub(crate) fn check() -> Result<(), String> {
    let adrs = read_dir(Path::new(ADR_DIR), "adr")?;
    let pms = read_dir(Path::new(PM_DIR), "postmortem")?;
    let mut problems: Vec<String> = Vec::new();
    for record in &adrs {
        check_record(record, &ADR_SECTIONS, &mut problems);
        check_status(record, &adrs, &mut problems);
    }
    for record in &pms {
        check_record(record, &PM_SECTIONS, &mut problems);
    }
    check_index(Path::new(ADR_DIR), &adrs, &mut problems);
    check_index(Path::new(PM_DIR), &pms, &mut problems);
    if problems.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{} problema(s) de governança: {}",
        problems.len(),
        problems.join(" · ")
    ))
}

/// Lê os registos `NNNN-slug.md` de `dir` (o `README.md` é o índice), em ordem.
fn read_dir(dir: &Path, kind: &str) -> Result<Vec<Record>, String> {
    let entries =
        fs::read_dir(dir).map_err(|error| format!("falha ao ler {}: {error}", dir.display()))?;
    let mut records: Vec<Record> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("falha ao ler {}: {error}", dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name == "README.md" {
            continue;
        }
        let Some(id) = identifier(name, kind) else {
            return Err(format!(
                "{}: nome fora do padrão `{kind}-NNNN-slug.md` (ou `NNNN-slug.md`)",
                path.display()
            ));
        };
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("falha ao ler {}: {error}", path.display()))?;
        records.push(Record { id, path, text });
    }
    records.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(records)
}

/// Extrai o id de um nome de ficheiro (`0016-slug.md` ou `PM-0002-slug.md`).
fn identifier(name: &str, kind: &str) -> Option<String> {
    let stem = name.strip_suffix(".md")?;
    let (prefix, rest) = stem.split_once('-')?;
    if prefix.len() == 4 && prefix.chars().all(|c| c.is_ascii_digit()) {
        return Some(prefix.to_owned());
    }
    if kind == "postmortem" && prefix == "PM" {
        let (digits, _) = rest.split_once('-')?;
        if digits.len() == 4 && digits.chars().all(|c| c.is_ascii_digit()) {
            return Some(format!("PM-{digits}"));
        }
    }
    None
}

/// Verifica título, secções obrigatórias e ids duplicados.
fn check_record(record: &Record, sections: &[&str], problems: &mut Vec<String>) {
    let name = record.path.display();
    let title_ok = record
        .text
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.starts_with(&format!("# {} — ", record.id)));
    if !title_ok {
        problems.push(format!(
            "{name}: título deve ser `# {} — <título>`",
            record.id
        ));
    }
    for section in sections {
        if !record.text.contains(section) {
            problems.push(format!("{name}: falta a secção `{section}`"));
        }
    }
}

/// Verifica `Status`/`Supersedes`/`Superseded by` e a simetria da substituição.
fn check_status(record: &Record, adrs: &[Record], problems: &mut Vec<String>) {
    let name = record.path.display();
    let Some(status) = metadata(&record.text, "Status") else {
        problems.push(format!("{name}: falta `**Status:**`"));
        return;
    };
    if !matches!(status.as_str(), "aceite" | "substituída" | "proposta") {
        problems.push(format!(
            "{name}: status `{status}` (use aceite|substituída|proposta)"
        ));
    }
    let supersedes = metadata(&record.text, "Supersedes").unwrap_or_default();
    let superseded_by = metadata(&record.text, "Superseded by").unwrap_or_default();
    let referenced = references(&supersedes);
    for reference in referenced.iter().chain(references(&superseded_by).iter()) {
        if !adrs.iter().any(|adr| &adr.id == reference) {
            problems.push(format!("{name}: referencia `{reference}`, que não existe"));
        }
    }
    // Simetria: as duas pontas da substituição têm de concordar.
    for reference in &referenced {
        let symmetric = adrs
            .iter()
            .find(|adr| &adr.id == reference)
            .and_then(|adr| metadata(&adr.text, "Superseded by"))
            .is_some_and(|value| references(&value).contains(&record.id));
        if !symmetric {
            problems.push(format!(
                "{name}: substitui `{reference}`, mas `{reference}` não aponta de volta"
            ));
        }
    }
    for reference in &references(&superseded_by) {
        let symmetric = adrs
            .iter()
            .find(|adr| &adr.id == reference)
            .and_then(|adr| metadata(&adr.text, "Supersedes"))
            .is_some_and(|value| references(&value).contains(&record.id));
        if !symmetric {
            problems.push(format!(
                "{name}: é substituída por `{reference}`, mas `{reference}` não o diz"
            ));
        }
    }
    if status == "substituída" && references(&superseded_by).is_empty() {
        problems.push(format!("{name}: `substituída` exige `Superseded by`"));
    }
    if status == "aceite" && !references(&superseded_by).is_empty() {
        problems.push(format!("{name}: `aceite` não pode ter `Superseded by`"));
    }
}

/// Valor de uma linha `**<campo>:** valor` (aceita o marcador de lista `- `).
fn metadata(text: &str, field: &str) -> Option<String> {
    let prefix = format!("**{field}:**");
    text.lines().find_map(|line| {
        let line = line.trim();
        let line = line.strip_prefix("- ").unwrap_or(line);
        line.strip_prefix(&prefix)
            .map(|value| value.trim().to_owned())
    })
}

/// Ids (`NNNN`) citados num valor de metadado; `—` (ou vazio) não cita nada.
fn references(value: &str) -> Vec<String> {
    value
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| token.len() == 4 && token.chars().all(|c| c.is_ascii_digit()))
        .map(str::to_owned)
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect()
}

/// O `README.md` do directório lista todos os registos (índice sem deriva).
fn check_index(dir: &Path, records: &[Record], problems: &mut Vec<String>) {
    let path = dir.join("README.md");
    let Ok(index) = fs::read_to_string(&path) else {
        problems.push(format!("{}: falta o índice", path.display()));
        return;
    };
    for record in records {
        if !index.contains(&record.id) {
            problems.push(format!("{}: não lista `{}`", path.display(), record.id));
        }
    }
}
