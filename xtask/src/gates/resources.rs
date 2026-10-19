//! Gate **G18** — recursos globais e piso visível (Fase 3 / F3, M8).
//!
//! Prova, no **binário**: a precedência é declarada (local ganha por nome em skills e
//! receitas; guardrails e plugins somam) e **tudo o que o runtime usa é visível** — cada
//! listagem diz de onde vem cada recurso efectivo.

use super::support::{Ctx, Gate, read, write};

/// Regra declarativa mínima.
fn rule(id: &str, fact: &str, matcher: &str) -> String {
    format!(
        "---\nid: {id}\nversion: 1\nfact: {fact}\nmatch: '{matcher}'\ndecision: deny\nreason: teste\n---\n"
    )
}

/// Um `SKILL.md` mínimo.
fn skill(description: &str, body: &str) -> String {
    format!("---\nname: x\ndescription: {description}\n---\n\n{body}\n")
}

/// Um plugin `observe` mínimo (script `sh` que responde `pass`).
fn plugin(name: &str) -> (String, &'static str) {
    (
        format!(
            "name = \"{name}\"\nentrypoint = \"./run.sh\"\nevents = [\"user.input\"]\nmode = \"observe\"\ntimeout_ms = 2000\n"
        ),
        "#!/bin/sh\nread -r line\nprintf '{\"verdict\":\"pass\"}\\n'\n",
    )
}

/// Corre o gate.
pub(crate) fn g18(ctx: &Ctx) -> Result<Gate, String> {
    let mut gate = Gate::new("G18", "Recursos globais e piso visivel (M8)");
    let dir = ctx.fresh("g18")?;

    // ── Os recursos nos **dois** níveis (o global vive em `<dir>/xdg/ktu/`) ──────
    write(
        &dir.join("xdg/ktu/skills/rust/SKILL.md"),
        &skill("rust global", "protocolo global"),
    )?;
    write(
        &dir.join("xdg/ktu/skills/kd/SKILL.md"),
        &skill("kd global", "protocolo kd"),
    )?;
    write(
        &dir.join(".agents/skills/rust/SKILL.md"),
        &skill("rust local", "protocolo local"),
    )?;
    write(
        &dir.join("xdg/ktu/guardrails/deny-env.md"),
        &rule("deny-env", "path", "*.env"),
    )?;
    write(
        &dir.join(".agents/guardrails/deny-bash.md"),
        &rule("deny-bash", "tool", "bash"),
    )?;
    write(
        &dir.join("xdg/ktu/recipes/revisao.md"),
        "---\nname: revisao\ndescription: global\nmax_steps: 9\n---\n\nreve global\n",
    )?;
    write(
        &dir.join("xdg/ktu/recipes/so-global.md"),
        "---\nname: so-global\ndescription: so global\n---\n\ncorre\n",
    )?;
    write(
        &dir.join(".agents/commands/revisao.md"),
        "---\nname: revisao\ndescription: local\nmax_steps: 2\n---\n\nreve local\n",
    )?;
    for (base, name) in [
        ("xdg/ktu/plugins", "g1"),
        ("xdg/ktu/plugins", "g2"),
        (".ktu/plugins", "g1"),
    ] {
        let (manifest, script) = plugin(name);
        write(&dir.join(format!("{base}/{name}/plugin.toml")), &manifest)?;
        write(&dir.join(format!("{base}/{name}/run.sh")), script)?;
        let path = dir.join(format!("{base}/{name}/run.sh"));
        let chmod = super::support::exec(
            std::path::Path::new("chmod"),
            &["+x", &path.to_string_lossy()],
            &dir,
            "",
        )?;
        if !chmod.status.success() {
            return Err("chmod do plugin falhou".to_owned());
        }
    }

    // ── 1. Skills: local ganha por nome; a origem aparece ──────────────────────
    let skills = ctx.ktu_ok(&dir, &["skills", "ls"], "")?;
    gate.check(
        "skills: o local ganha por nome (uma so `rust`)",
        skills.contains("rust\trust local\tlocal") && !skills.contains("rust global"),
        skills.lines().take(3).collect::<Vec<_>>().join(" | "),
    );
    gate.check(
        "skills: a global sem homonimo continua visivel, com origem",
        skills.contains("kd\tkd global\tglobal"),
        "kd global".to_owned(),
    );

    // ── 2. Receitas: local ganha por nome; a origem aparece ───────────────────
    let commands = ctx.ktu_ok(&dir, &["commands", "ls"], "")?;
    gate.check(
        "receitas: o local ganha por nome e a origem aparece",
        commands.contains("revisao\tlocal\tlocal")
            && commands.contains("so-global\tso global\tglobal"),
        commands.replace('\n', " | "),
    );

    // ── 3. Guardrails: o piso **soma** e o `list` traz a origem de todas ──────
    let rules = ctx.ktu_ok(&dir, &["guardrails", "list", "--json"], "")?;
    let value: serde_json::Value =
        serde_json::from_str(&rules).map_err(|error| format!("json invalido: {error}"))?;
    let listed: Vec<(String, String)> = value
        .get("data")
        .and_then(|data| data.get("rules"))
        .and_then(|rules| rules.as_array())
        .map(|rules| {
            rules
                .iter()
                .filter_map(|rule| {
                    Some((
                        rule.get("id")?.as_str()?.to_owned(),
                        rule.get("origin")?.as_str()?.to_owned(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    gate.check(
        "guardrails: o piso global vem primeiro, com origem",
        listed
            == vec![
                ("deny-env".to_owned(), "global".to_owned()),
                ("deny-bash".to_owned(), "local".to_owned()),
            ],
        format!("{listed:?}"),
    );
    gate.check(
        "M8: nenhuma regra efectiva fica sem origem",
        listed.len() == 2 && listed.iter().all(|(_, origin)| !origin.is_empty()),
        format!("{listed:?}"),
    );

    // M5 ao vivo: a regra global nega `.env` mesmo com regras locais no projecto.
    let script = dir.join("g18.jsonl");
    write(
        &script,
        "{\"text\":\"\",\"tool_calls\":[{\"id\":\"c1\",\"name\":\"read_file\",\"arguments\":\"{\\\"path\\\":\\\"segredos.env\\\"}\"}]}\n{\"text\":\"fim\"}\n",
    )?;
    write(&dir.join("segredos.env"), "CHAVE=1\n")?;
    let m5 = ctx.ktu(
        &dir,
        &[
            "run",
            "le segredos.env",
            "--provider",
            "scripted",
            "--script",
            &script.to_string_lossy(),
            "--session",
            "m5",
            "--max-steps",
            "2",
        ],
        "",
    )?;
    let log = read(&dir.join(".ktu/sessions/m5.jsonl")).unwrap_or_default();
    gate.check(
        "M5: o projecto nao desliga a regra global",
        m5.status.success() && log.contains("\"allowed\":false"),
        format!("exit={}", m5.status),
    );

    // ── 4. Plugins: aditivos, com a origem de cada um ────────────────────────
    let plugins = ctx.ktu_ok(&dir, &["plugins", "list"], "")?;
    let g1_global = plugins.contains("g1\tglobal");
    let g1_local = plugins.contains("g1\tlocal");
    gate.check(
        "plugins: aditivos (o mesmo nome nos dois niveis corre duas vezes)",
        g1_global && g1_local && plugins.contains("g2\tglobal"),
        plugins.replace('\n', " | "),
    );

    // ── 5. O prompt compõe as skills **efectivas** (local ganha) ─────────────
    let prompt = ctx.ktu_ok(&dir, &["prompt"], "")?;
    gate.check(
        "o prompt compõe as skills efectivas (local ganha)",
        prompt.contains("# Skills")
            && prompt.contains("## rust — rust local")
            && prompt.contains("protocolo local")
            && !prompt.contains("protocolo global"),
        format!("tem Skills: {}", prompt.contains("# Skills")),
    );
    gate.check(
        "o prompt compõe tambem a skill so global (por nome)",
        prompt.contains("## kd — kd global"),
        "kd no prompt".to_owned(),
    );

    // ── 6. A precedencia vale no **runtime** (receita local impõe a config) ──
    let _run = ctx.ktu(
        &dir,
        &[
            "run",
            "/revisao",
            "--provider",
            "mock",
            "--session",
            "receita",
        ],
        "",
    )?;
    let log = read(&dir.join(".ktu/sessions/receita.jsonl")).unwrap_or_default();
    gate.check(
        "receitas: o `max_steps` efectivo e o da receita local",
        log.matches("\"event\":\"assistant\"").count() == 2,
        format!("passos={}", log.matches("\"event\":\"assistant\"").count()),
    );

    gate.artifact("skills.txt", skills);
    gate.artifact("plugins.txt", plugins);
    gate.artifact("guardrails.json", rules);
    gate.artifact("prompt.txt", prompt);
    gate.note(
        "`ktu agents ls` **nao** ganhou origem: os agentes declarativos nao tem nivel global \
         (uma coluna constante seria ruido); o M8 lista guardrails, skills, plugins e receitas."
            .to_owned(),
    );
    Ok(gate)
}
