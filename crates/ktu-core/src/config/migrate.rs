//! Versão do schema de configuração e migrações (F5).
//!
//! O documento declara a sua versão (`schema = 1`); a **ausência** significa **v0**
//! (o formato anterior, sem versão). As duas variantes existem de propósito:
//!
//! - [`read_migrate`] — **leitura**: o documento na versão actual **em memória**,
//!   que **nunca remove chaves** (outro chamador pode ainda consultá-las);
//! - [`migrate`] — **escrita**: o documento a persistir; pode **remover** as chaves
//!   que um passo declare obsoletas e diz se mudou algo (idempotência).
//!
//! Uma versão **futura** é **fail-closed**: erro legível e nenhuma escrita — nunca
//! se "migra para trás" sobre um ficheiro que não se sabe ler.

use toml::Table;
use toml::Value;

use super::{DEFERRED_TOOLS, SCHEMA, SCHEMA_KEY};

/// Um passo de migração: a versão de destino e as chaves que ele **retira**.
///
/// `pub(super)`: o mecanismo de retirada é exercitado pelos testes do módulo com um
/// passo de ensaio (a lista real está vazia), sem entrar na API pública.
#[derive(Clone, Copy)]
pub(super) struct Step {
    /// Versão que o passo produz.
    pub(super) to: u32,
    /// Chaves obsoletas que o passo retira (a escrita pode retirar; a leitura não).
    pub(super) deprecated: &'static [&'static str],
}

/// O que uma migração faz com as chaves que um passo declara obsoletas.
///
/// (Enum e não `bool`: o projecto proíbe booleanos em parâmetros — um `false` não
/// diz o que faz.)
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Obsolete {
    /// A leitura **não** as retira: outro chamador pode ainda consultá-las.
    Keep,
    /// A escrita retira-as: o ficheiro fica exactamente na versão actual.
    Drop,
}

/// Passos de migração, por ordem crescente de versão.
///
/// O **v1** normaliza `deferred_tools` (a forma plana/CSV do v0 → lista) e carimba a
/// versão; a lista de chaves obsoletas está **vazia** porque nenhuma chave foi
/// retirada desde o início (as [`KEYS`] nunca mudaram — verificado no histórico).
/// O mecanismo fica montado para o v2: retirar uma chave é acrescentá-la aqui.
const STEPS: &[Step] = &[Step {
    to: 1,
    deprecated: &[],
}];

/// Interpreta um documento TOML (a tabela crua, antes de tipar).
///
/// # Errors
/// Devolve mensagem legível se o TOML for inválido.
pub fn table(text: &str) -> Result<Table, String> {
    toml::from_str(text).map_err(|error| format!("config invalida: {error}"))
}

/// Versão do schema declarada num documento (ausente ou inválida ⇒ **0**, o v0).
#[must_use]
pub fn schema_of(table: &Table) -> u32 {
    table
        .get(SCHEMA_KEY)
        .and_then(Value::as_integer)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0)
}

/// Migração de **leitura**: o documento na versão actual, **em memória**.
///
/// Acrescenta o que falta e normaliza valores das formas antigas, mas **nunca
/// remove chaves** — quem lê não sabe quem mais as consulta. Uma versão futura é
/// erro (fail-closed).
///
/// # Errors
/// Devolve mensagem legível se a versão declarada for mais recente que [`SCHEMA`].
pub fn read_migrate(mut table: Table) -> Result<Table, String> {
    check_version(&table)?;
    let _changed = apply_steps(&mut table, Obsolete::Keep, STEPS);
    Ok(table)
}

/// Migração de **escrita**: o documento a persistir, na versão actual.
///
/// Faz o mesmo que [`read_migrate`] e **pode** retirar as chaves que um passo declare
/// obsoletas. Devolve `(documento, mudou)`; `mudou == false` ⇒ já estava migrado
/// (correr duas vezes não altera nada).
///
/// # Errors
/// Devolve mensagem legível se a versão declarada for mais recente que [`SCHEMA`].
pub fn migrate(mut table: Table) -> Result<(Table, bool), String> {
    check_version(&table)?;
    let changed = apply_steps(&mut table, Obsolete::Drop, STEPS);
    Ok((table, changed))
}

/// Recusa uma versão mais recente que a do binário (fail-closed, sem escrever).
fn check_version(table: &Table) -> Result<(), String> {
    let found = schema_of(table);
    if found > SCHEMA {
        return Err(format!(
            "config no schema v{found}, mais recente que o v{SCHEMA} deste binario: \
             atualize o ktu (nenhum ficheiro foi alterado)"
        ));
    }
    Ok(())
}

/// Aplica os passos em falta; com [`Obsolete::Drop`], retira também as obsoletas.
///
/// Os passos são um parâmetro (e não a constante [`STEPS`] directa) para que o
/// mecanismo de retirada possa ser exercitado por um passo de ensaio — hoje a lista
/// real está vazia.
pub(super) fn apply_steps(table: &mut Table, obsolete: Obsolete, steps: &[Step]) -> bool {
    let mut changed = false;
    for step in steps {
        if schema_of(table) >= step.to {
            continue;
        }
        if obsolete == Obsolete::Drop {
            for key in step.deprecated {
                let _removed = table.remove(*key);
            }
        }
        normalise(table);
        let _previous = table.insert(SCHEMA_KEY.to_owned(), Value::Integer(i64::from(step.to)));
        changed = true;
    }
    changed
}

/// Normaliza os valores que mudaram de forma no v1 (string CSV → lista).
fn normalise(table: &mut Table) {
    let Some(Value::String(csv)) = table.get(DEFERRED_TOOLS) else {
        return;
    };
    let tools: Vec<Value> = csv
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| Value::String(name.to_owned()))
        .collect();
    let _previous = table.insert(DEFERRED_TOOLS.to_owned(), Value::Array(tools));
}
