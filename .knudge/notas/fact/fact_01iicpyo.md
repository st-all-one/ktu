---
id: fact_01iicpyo
type: fact
statement: H6 concluida: permissoes por tool (Always/Ask/Never) + politica como dado (escalate/remedy); steering adiado
created_at: 2026-10-18T00:40:25.576Z
body_hash: 6619a276
schema_version: 2
anchors: [crates/ktu-core/src/permission.rs, crates/ktu-core/src/guard.rs, crates/ktu-core/src/guardrails.rs, crates/ktu-cli/src/permission_cmd.rs]
---
H6 (task_01s9n99d, parcial) concluida. (1) Permissoes por tool: novo ktu-core::permission (Permission Always/Ask/Never; Store em BTreeMap; .agents/permissions.toml, project-bound => sobrevive a resume). PermissionGuard e o ULTIMO elo da cadeia (structural -> declarative -> permission): Always=Allow, Ask=Escalate, Never=Deny(DenyReason::Policy) - nunca alarga (I16). CLI ktu permission ls|set|unset (envelope --json). (2) Politica como dado (seam): Guard ganhou remedy(&Fact) com default estrutural; Structural/Declarative/Permission/Chain implementam-no. O guardrail declarativo aceita decision: deny|escalate e remedy proprio (Rule.decision: RuleDecision, Rule.remedy). O seam Guard::evaluate sobre Fact mantem-se o unico caminho (M2). Verificado por integracao: never -> "negado: policy - ..."; ask -> "requer aprovacao: aprove a tool ..."; regra escalate/deny com remedy proprio; permissao sobrevive a resume. +5 unit (permission) +3 integracao (tests/permission.rs). Adiado A7 (task_01hqpcjw): steering/admissao duravel - sem consumidor (run one-shot; REPL sincrono; serve/TUI recusaria o 2.o turno) e exigiria evento novo no log. Gate: make check + typos + machete verdes.