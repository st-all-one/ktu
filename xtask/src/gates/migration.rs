//! Gate **G20** — migrações de configuração (Fase 3 / F5).
//!
//! Prova, pelo **binário**, que uma config antiga (v0, sem versão) é lida sem ser
//! reescrita, que a escrita migra **uma vez** (idempotente) e carimba a versão, e que
//! uma versão **futura** é fail-closed: erro legível e **nenhuma** escrita.

use super::support::{Ctx, Gate, hash, read, stdout, write};

/// Documento **v0**: sem versão e com a forma plana (CSV) de `deferred_tools`.
const V0: &str = "model = \"m\"\ndeferred_tools = \"bash, write_file\"\n";

/// Conteúdo do `config.toml` local (ausente ⇒ vazio).
fn local(dir: &std::path::Path) -> String {
    read(&dir.join(".agents/config.toml")).unwrap_or_default()
}

/// Corre o gate.
pub(crate) fn g20(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new(
        "G20",
        "Migracoes de config (schema, idempotencia, fail-closed)",
    );
    let dir = ctx.fresh("g20")?;
    // Os dois níveis começam em v0 (o global dentro do XDG isolado do gate).
    write(&dir.join(".agents/config.toml"), V0)?;
    write(&dir.join("xdg/ktu/config.toml"), V0)?;

    // ── 1. O diagnóstico diz a versão e se migraria — sem escrever ────────────
    let before = local(&dir);
    let doctor = ctx.ktu_ok(&dir, &["config", "doctor", "--json"], "")?;
    gate.check(
        "o doctor mostra a versao dos dois niveis e se migrariam",
        doctor.contains("\"schema\":0")
            && doctor.matches("\"migrates\":true").count() == 2
            && doctor.contains("\"level\":\"global\""),
        format!(
            "local e global em v0: {}",
            doctor.matches("\"schema\":0").count()
        ),
    );
    gate.check(
        "o doctor nao escreve",
        local(&dir) == before,
        format!("hash {}", hash(&local(&dir))),
    );

    // ── 2. A leitura normaliza em memória e **nao** altera o ficheiro ────────
    let tools = ctx.ktu_ok(&dir, &["config", "get", "deferred_tools"], "")?;
    gate.check(
        "a leitura aceita a forma plana antiga (CSV) sem reescrever o ficheiro",
        tools.trim() == "bash,write_file" && local(&dir) == before,
        format!("valor {tools:?} com o ficheiro intacto"),
    );

    // ── 3. A escrita migra uma vez e carimba a versao (idempotente) ──────────
    let first = ctx.ktu(&dir, &["config", "set", "--local", "effort", "high"], "")?;
    let stderr = String::from_utf8_lossy(&first.stderr).into_owned();
    let migrated = local(&dir);
    gate.check(
        "a escrita migra com aviso em stderr (nunca em silencio)",
        first.status.success()
            && stderr.contains("migrada do schema v0 para v1")
            && migrated.contains("schema = 1")
            && migrated.contains("[\"bash\", \"write_file\"]"),
        format!("schema carimbado: {}", migrated.contains("schema = 1")),
    );
    let second = ctx.ktu(&dir, &["config", "set", "--local", "model", "m"], "")?;
    let again = ctx.ktu_ok(&dir, &["config", "doctor", "--json"], "")?;
    gate.check(
        "migrar duas vezes nao muda nada (idempotente)",
        second.status.success() && local(&dir) == migrated && again.contains("\"migrates\":false"),
        format!("hash {} -> {}", hash(&migrated), hash(&local(&dir))),
    );

    // ── 4. Versao futura: erro legivel e nenhuma escrita (fail-closed) ───────
    let future = "schema = 3\nmodel = \"m\"\n";
    write(&dir.join(".agents/config.toml"), future)?;
    let refused = ctx.ktu(&dir, &["config", "set", "--local", "model", "outro"], "")?;
    let refusal = format!(
        "{}{}",
        stdout(&refused),
        String::from_utf8_lossy(&refused.stderr)
    );
    gate.check(
        "uma versao futura e recusada com mensagem legivel",
        !refused.status.success()
            && refusal.contains("schema v3")
            && refusal.contains("nenhum ficheiro foi alterado"),
        refusal
            .lines()
            .find(|line| line.contains("schema v3"))
            .unwrap_or_default()
            .trim()
            .to_owned(),
    );
    gate.check(
        "a recusa nao escreve",
        local(&dir) == future,
        format!("hash {}", hash(&local(&dir))),
    );

    gate.artifact("doctor-v0.json", doctor);
    gate.artifact("config-antes.toml", before.clone());
    gate.artifact("config-depois.toml", migrated.clone());
    gate.artifact("config-recusada.toml", local(&dir));
    gate.artifact(
        "hashes.txt",
        format!("{}\n{}\n", hash(&before), hash(&migrated)),
    );
    gate.note(
        "O v1 normaliza `deferred_tools` (a forma plana/CSV do v0 -> lista) e carimba \
         `schema = 1`; a lista de chaves obsoletas esta **vazia** (nenhuma chave foi \
         retirada desde o inicio — as KEYS nunca mudaram), pelo que o mecanismo de \
         retirada fica montado para o v2 (testado por um passo de ensaio no core). \
         Leitura e escrita sao diferentes de forma deliberada: `read_migrate` nunca retira \
         chaves (outro chamador pode consulta-las), `migrate` pode."
            .to_owned(),
    );
    Ok(gate)
}
