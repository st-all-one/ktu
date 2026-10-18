//! Tool calls **declaradas como texto** e reparação de input guiada pelo schema.
//!
//! Modelos locais sem *tool calling* nativo escrevem a chamada no `content`
//! (bloco JSON ou `<tool_call>`). Decodificamos contra o **catálogo fechado**
//! ([`ToolSpec`]): um nome fora do catálogo é ignorado (*fail-closed*). A
//! reparação de argumentos só actua quando é **inequívoca** (profundidade ≤ 6).

use crate::domain::{ToolCall, ToolCallId, ToolSpec};
use serde_json::{Map, Value};

/// Profundidade máxima da reparação.
const MAX_DEPTH: u8 = 6;

/// Decodifica tool calls declaradas em `text` contra `specs`.
///
/// Devolve `None` quando não há nenhuma chamada válida (o texto é resposta normal).
#[must_use]
pub(super) fn decode(text: &str, specs: &[ToolSpec]) -> Option<Vec<ToolCall>> {
    let mut calls = Vec::new();
    for value in candidates(text) {
        match value {
            Value::Array(items) => {
                for item in &items {
                    push_call(&mut calls, item, specs);
                }
            }
            other => push_call(&mut calls, &other, specs),
        }
    }
    (!calls.is_empty()).then_some(calls)
}

/// Repara `arguments` (JSON em texto) contra o JSON Schema de `spec`.
///
/// Sem schema conhecido devolve o texto inalterado.
#[must_use]
pub(super) fn repair_arguments(arguments: &str, spec: Option<&ToolSpec>) -> String {
    let Some(spec) = spec else {
        return arguments.to_owned();
    };
    let Ok(schema) = serde_json::from_str::<Value>(&spec.parameters) else {
        return arguments.to_owned();
    };
    let Ok(value) = serde_json::from_str::<Value>(arguments) else {
        return arguments.to_owned();
    };
    repair(&value, &schema, 0).to_string()
}

/// Repara um valor contra um JSON Schema (só quando inequívoco).
#[must_use]
pub(super) fn repair(value: &Value, schema: &Value, depth: u8) -> Value {
    if depth >= MAX_DEPTH {
        return value.clone();
    }
    let next = depth.saturating_add(1);
    match schema.get("type").and_then(Value::as_str) {
        Some("object") => repair_object(value, schema, next),
        Some("array") => repair_array(value, schema, next),
        Some("integer") => coerce_integer(value),
        Some("number") => coerce_number(value),
        Some("boolean") => coerce_bool(value),
        _ => value.clone(),
    }
}

/// Repara um objecto: `{"...": "json"}` em string e propriedades aninhadas.
fn repair_object(value: &Value, schema: &Value, depth: u8) -> Value {
    let Some(object) = as_object(value) else {
        return value.clone();
    };
    let properties = schema.get("properties").and_then(Value::as_object);
    let mut out = Map::new();
    for (key, item) in object {
        let repaired = match properties.and_then(|map| map.get(key.as_str())) {
            Some(property) => repair(&item, property, depth),
            None => item,
        };
        out.insert(key, repaired);
    }
    Value::Object(out)
}

/// Repara um array: `"[...]"` em string e itens aninhados.
fn repair_array(value: &Value, schema: &Value, depth: u8) -> Value {
    let Some(items) = as_array(value) else {
        return value.clone();
    };
    let item_schema = schema.get("items");
    let repaired = items
        .iter()
        .map(|item| match item_schema {
            Some(schema) => repair(item, schema, depth),
            None => item.clone(),
        })
        .collect();
    Value::Array(repaired)
}

/// Repara uma string que contenha um objecto/array JSON (só quando inequívoco).
fn as_object(value: &Value) -> Option<Map<String, Value>> {
    match value {
        Value::Object(map) => Some(map.clone()),
        Value::String(text) => match serde_json::from_str::<Value>(text.trim()) {
            Ok(Value::Object(map)) => Some(map),
            _ => None,
        },
        _ => None,
    }
}

/// Repara uma string que contenha um array JSON (só quando inequívoco).
fn as_array(value: &Value) -> Option<Vec<Value>> {
    match value {
        Value::Array(items) => Some(items.clone()),
        Value::String(text) => match serde_json::from_str::<Value>(text.trim()) {
            Ok(Value::Array(items)) => Some(items),
            _ => None,
        },
        _ => None,
    }
}

/// Repara uma string num inteiro (`"20"` → `20`), com recurso a real.
fn coerce_integer(value: &Value) -> Value {
    let Value::String(text) = value else {
        return value.clone();
    };
    let trimmed = text.trim();
    if let Ok(number) = trimmed.parse::<i64>() {
        return Value::from(number);
    }
    match trimmed.parse::<f64>() {
        Ok(number) => Value::from(number),
        Err(_) => value.clone(),
    }
}

/// Repara uma string num número real (`"1.5"` → `1.5`).
fn coerce_number(value: &Value) -> Value {
    let Value::String(text) = value else {
        return value.clone();
    };
    match text.trim().parse::<f64>() {
        Ok(number) => Value::from(number),
        Err(_) => value.clone(),
    }
}

/// Repara uma string booleana num booleano (`"true"` → `true`).
fn coerce_bool(value: &Value) -> Value {
    match value {
        Value::String(text) => match text.trim() {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => value.clone(),
        },
        _ => value.clone(),
    }
}

/// Acrescenta a chamada se o nome estiver no catálogo fechado.
fn push_call(calls: &mut Vec<ToolCall>, value: &Value, specs: &[ToolSpec]) {
    let id = format!("declared-{}", calls.len().saturating_add(1));
    if let Some(call) = to_call(value, specs, id) {
        calls.push(call);
    }
}

/// Converte um valor num `ToolCall`, exigindo o nome no catálogo.
fn to_call(value: &Value, specs: &[ToolSpec], id: String) -> Option<ToolCall> {
    let object = value.as_object()?;
    let name = object
        .get("name")
        .or_else(|| object.get("tool"))
        .or_else(|| object.get("tool_name"))?
        .as_str()?;
    if !specs.iter().any(|spec| spec.name == name) {
        return None;
    }
    let arguments = object
        .get("arguments")
        .or_else(|| object.get("args"))
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let arguments = match arguments {
        Value::String(text) => serde_json::from_str(&text).unwrap_or(Value::String(text)),
        other => other,
    };
    Some(ToolCall {
        id: ToolCallId::new(id),
        name: name.to_owned(),
        arguments: arguments.to_string(),
    })
}

/// Valores JSON candidatos: blocos `<tool_call>`, blocos cercados e o texto todo.
fn candidates(text: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for block in tagged(text, "<tool_call>", "</tool_call>") {
        push_json(&mut out, block);
    }
    for block in fenced(text) {
        push_json(&mut out, block);
    }
    push_json(&mut out, text);
    out
}

/// Blocos entre `begin` e `end`.
fn tagged<'a>(text: &'a str, begin: &str, end: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(begin) {
        let after = rest.get(start.saturating_add(begin.len())..).unwrap_or("");
        let Some(stop) = after.find(end) else {
            break;
        };
        out.push(after.get(..stop).unwrap_or(""));
        rest = after.get(stop.saturating_add(end.len())..).unwrap_or("");
    }
    out
}

/// Blocos de código cercados por três crases (salta a linguagem da cerca).
fn fenced(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("```") {
        let after = rest.get(start.saturating_add(3)..).unwrap_or("");
        let body = after.split_once('\n').map_or(after, |(_lang, body)| body);
        let Some(stop) = body.find("```") else {
            break;
        };
        out.push(body.get(..stop).unwrap_or(""));
        rest = body.get(stop.saturating_add(3)..).unwrap_or("");
    }
    out
}

/// Acrescenta o valor se `text` for JSON válido.
fn push_json(out: &mut Vec<Value>, text: &str) {
    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        out.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, repair, repair_arguments};
    use crate::domain::ToolSpec;
    use serde_json::json;

    fn specs() -> Vec<ToolSpec> {
        vec![
            ToolSpec {
                name: "read_file".to_owned(),
                description: String::new(),
                parameters:
                    r#"{"type":"object","properties":{"path":{"type":"string"},"offset":{"type":"integer"}}}"#
                        .to_owned(),
            },
            ToolSpec {
                name: "bash".to_owned(),
                description: String::new(),
                parameters:
                    r#"{"type":"object","properties":{"command":{"type":"string"}}}"#.to_owned(),
            },
        ]
    }

    #[test]
    fn decodes_a_bare_json_object() {
        let calls = decode(
            r#"{"name":"read_file","arguments":{"path":"a.txt"}}"#,
            &specs(),
        )
        .unwrap_or_default();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls.first().map(|call| call.name.as_str()),
            Some("read_file")
        );
        assert!(
            calls
                .first()
                .is_some_and(|call| call.arguments.contains("a.txt"))
        );
    }

    #[test]
    fn decodes_a_tagged_block_and_ignores_unknown_tools() {
        let text = "<tool_call>{\"name\":\"rm\",\"arguments\":{}}</tool_call>";
        assert!(decode(text, &specs()).is_none());
    }

    #[test]
    fn ignores_plain_text() {
        assert!(decode("ola, tudo bem?", &specs()).is_none());
    }

    #[test]
    fn decodes_a_fenced_array_of_calls() {
        let text = "```json\n[{\"name\":\"bash\",\"arguments\":{\"command\":\"ls\"}}]\n```";
        let calls = decode(text, &specs()).unwrap_or_default();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls.first().map(|call| call.name.as_str()), Some("bash"));
    }

    #[test]
    fn repairs_an_object_in_a_string() {
        let schema = json!({"type": "object", "properties": {"a": {"type": "string"}}});
        let value = json!("{\"a\": \"x\"}");
        assert_eq!(repair(&value, &schema, 0), json!({"a": "x"}));
    }

    #[test]
    fn repairs_scalar_types_by_schema() {
        let schema = json!({"type": "object", "properties": {"n": {"type": "integer"}, "b": {"type": "boolean"}}});
        let value = json!({"n": "20", "b": "true"});
        assert_eq!(repair(&value, &schema, 0), json!({"n": 20, "b": true}));
    }

    #[test]
    fn repair_arguments_uses_the_tool_schema() {
        let specs = specs();
        let spec = specs.first();
        let repaired = repair_arguments(r#"{"path":"a.txt","offset":"3"}"#, spec);
        assert!(repaired.contains("\"offset\":3"), "{repaired}");
    }
}
