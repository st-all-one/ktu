//! `tool_search`: descobre tools **diferidas** e carrega-as para o próximo pedido.
//!
//! Um catálogo grande gasta contexto e confunde a escolha do modelo. A exposição é
//! **dado**: um [`ToolExecutor`](crate::tools::ToolExecutor) pode declarar tools
//! como *diferidas* (fora do catálogo); quando existem, o loop oferece `tool_search`,
//! que as ranqueia por **BM25** sobre nome/descrição (determinístico, `BTreeMap`-like
//! ordem estável) e as promove para o pedido seguinte — registado no log
//! (`tool.loaded`), para que *model-visible ⟺ logged*.
//!
//! É distinto de roteamento semântico: aqui o ganho é **não declarar** até ser
//! preciso, sem nunca esconder a tool do *enforcement* (o guard estrutural continua
//! a conhecer o conjunto completo do agente).

use std::cmp::Ordering;

use crate::domain::ToolSpec;

/// Nome canónico da tool de descoberta.
pub const TOOL: &str = "tool_search";

/// Quantas tools o `tool_search` devolve e promove, no máximo.
pub const TOP_K: usize = 8;

/// Texto devolvido quando a consulta não encontra nada.
pub const NO_MATCHES: &str = "[sem resultados]";

/// Parâmetro BM25 `k1` (saturação da frequência do termo).
const K1: f64 = 1.2;

/// Parâmetro BM25 `b` (normalização pelo comprimento do documento).
const B: f64 = 0.75;

/// Especificação da tool `tool_search`.
#[must_use]
pub fn spec() -> ToolSpec {
    ToolSpec {
        name: TOOL.to_owned(),
        description: "Procura nas tools disponiveis mas ainda nao declaradas (por nome ou \
                      descricao) e carrega as melhores para o proximo passo."
            .to_owned(),
        parameters:
            r#"{"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}"#
                .to_owned(),
    }
}

/// Interpreta os argumentos JSON da tool (`query` obrigatório).
///
/// # Errors
/// Devolve mensagem accionável se os argumentos faltarem ou forem malformados.
pub fn parse_query(arguments: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(arguments)
        .map_err(|error| format!("argumentos invalidos: {error}"))?;
    let query = value
        .get("query")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if query.trim().is_empty() {
        return Err("campo `query` em falta".to_owned());
    }
    Ok(query.trim().to_owned())
}

/// Ranqueia `specs` contra `query` por BM25 e devolve os índices, do melhor para o
/// pior (empates desfeitos pelo **nome**, para ser determinístico).
///
/// Devolve vazio quando a consulta não tem termos ou nada pontua.
#[must_use]
pub fn rank(query: &str, specs: &[ToolSpec], top_k: usize) -> Vec<usize> {
    let terms = unique(tokens(query));
    if terms.is_empty() || specs.is_empty() || top_k == 0 {
        return Vec::new();
    }
    let docs: Vec<Vec<String>> = specs.iter().map(document).collect();
    let total = as_f64(docs.len());
    let average = as_f64(docs.iter().map(Vec::len).sum::<usize>()) / total;
    let mut scored: Vec<(f64, &str, usize)> = Vec::new();
    for (index, doc) in docs.iter().enumerate() {
        let score = score(&terms, doc, &docs, average, total);
        if score > 0.0 {
            let name = specs.get(index).map_or("", |spec| spec.name.as_str());
            scored.push((score, name, index));
        }
    }
    scored.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.1.cmp(right.1))
    });
    scored
        .into_iter()
        .take(top_k)
        .map(|(_, _, index)| index)
        .collect()
}

/// Texto do resultado: as tools encontradas (nome, descrição e parâmetros) e
/// quantas continuam por carregar.
#[must_use]
pub fn render(found: &[ToolSpec], remaining: usize) -> String {
    if found.is_empty() {
        return format!("{NO_MATCHES} ({remaining} por carregar)");
    }
    let mut text = found
        .iter()
        .map(|spec| {
            format!(
                "{}: {}\nargumentos: {}",
                spec.name, spec.description, spec.parameters
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    text.push_str("\n\n(carregadas para o proximo passo)");
    text
}

/// Pontuação BM25 de um documento contra os termos da consulta.
fn score(terms: &[String], doc: &[String], docs: &[Vec<String>], average: f64, total: f64) -> f64 {
    let length = as_f64(doc.len());
    let mut score = 0.0;
    for term in terms {
        let frequency = doc.iter().filter(|token| *token == term).count();
        if frequency == 0 {
            continue;
        }
        let containing = as_f64(
            docs.iter()
                .filter(|other| other.iter().any(|token| token == term))
                .count(),
        );
        let idf = ((total - containing + 0.5) / (containing + 0.5)).ln_1p();
        let frequency = as_f64(frequency);
        let normalised = K1.mul_add(1.0 - B + B * length / average, frequency);
        let saturated = frequency * (K1 + 1.0) / normalised;
        score = idf.mul_add(saturated, score);
    }
    score
}

/// Converte uma contagem em `f64` para a pontuação BM25 (perda de precisão
/// irrelevante: são frequências e comprimentos de documentos pequenos).
#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "BM25 em f64; contagens de tokens nunca chegam perto de 2^52"
)]
fn as_f64(value: usize) -> f64 {
    value as f64
}

/// Tokens de um documento: o **nome pesa o dobro** da descrição.
fn document(spec: &ToolSpec) -> Vec<String> {
    let mut parts = tokens(&spec.name);
    parts.extend(tokens(&spec.name));
    parts.extend(tokens(&spec.description));
    parts
}

/// Tokeniza: minúsculas, separando por tudo o que não é alfanumérico.
fn tokens(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Remove repetições preservando a ordem de primeira ocorrência.
fn unique(items: Vec<String>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for item in items {
        if !seen.contains(&item) {
            seen.push(item);
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::{NO_MATCHES, TOOL, parse_query, rank, render, spec, tokens};
    use crate::domain::ToolSpec;

    fn tool(name: &str, description: &str) -> ToolSpec {
        ToolSpec {
            name: name.to_owned(),
            description: description.to_owned(),
            parameters: "{}".to_owned(),
        }
    }

    fn catalog() -> Vec<ToolSpec> {
        vec![
            tool("glob", "Encontra ficheiros por padrao de caminho"),
            tool("grep", "Procura um padrao no conteudo dos ficheiros"),
            tool("sql_query", "Executa uma consulta SQL numa base de dados"),
            tool("image_resize", "Redimensiona imagens locais"),
        ]
    }

    #[test]
    fn ranks_by_relevance_and_is_deterministic() {
        let specs = catalog();
        let found = rank("procura padrao nos ficheiros", &specs, 8);
        assert_eq!(found.first().copied(), Some(1), "grep deve ganhar");

        let again = rank("procura padrao nos ficheiros", &specs, 8);
        assert_eq!(found, again, "mesma consulta, mesma ordem");
    }

    #[test]
    fn respects_top_k_and_ignores_unrelated_queries() {
        let specs = catalog();
        assert_eq!(rank("glob", &specs, 1).len(), 1);
        assert_eq!(
            rank("palavra que nao existe", &specs, 8),
            Vec::<usize>::new()
        );
        assert_eq!(rank("", &specs, 8), Vec::<usize>::new());
        assert_eq!(rank("glob", &[], 8), Vec::<usize>::new());
        assert_eq!(rank("glob", &specs, 0), Vec::<usize>::new());
    }

    #[test]
    fn ties_are_broken_by_name() {
        let specs = vec![
            tool("zebra", "mesma descricao"),
            tool("alfa", "mesma descricao"),
        ];
        let found = rank("mesma descricao", &specs, 2);
        assert_eq!(found, vec![1, 0], "alfa antes de zebra");
    }

    #[test]
    fn parses_the_query_and_rejects_bad_arguments() {
        assert_eq!(
            parse_query(r#"{"query":"  imagens  "}"#).ok().as_deref(),
            Some("imagens")
        );
        assert!(parse_query(r#"{"query":""}"#).is_err());
        assert!(parse_query("{}").is_err());
        assert!(parse_query("nao json").is_err());
    }

    #[test]
    fn renders_the_found_tools_with_the_schema() {
        let specs = catalog();
        let found = vec![specs.first().cloned().unwrap_or_else(|| tool("x", "y"))];
        let text = render(&found, 3);
        assert!(text.contains("glob: Encontra ficheiros"));
        assert!(text.contains("argumentos: {}"));
        assert!(text.contains("carregadas para o proximo passo"));
        assert!(render(&[], 3).contains(NO_MATCHES));
    }

    #[test]
    fn the_spec_declares_the_query_parameter() {
        let spec = spec();
        assert_eq!(spec.name, TOOL);
        assert!(spec.parameters.contains("query"));
        assert!(tokens("read_File").contains(&"read".to_owned()));
    }
}
