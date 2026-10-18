# Motor de política completo (política como **dado**)

- **Estado:** **implementado** (H6, *seam*) — `Guard::evaluate`/`remedy` sobre `Fact`; guardrails com `deny|escalate` + `remedy`. O motor genérico (regras por combinação de factos) continua deferido.
- **Origem:** katu `wiki/_ref/plan/03-contrato-de-politica.md`, `policy/containment.toml`;
  `plan.md` §2.8/§2.12-B.
- **Âncora:** `crates/ktu-core/src/guard.rs`, `.agents/guardrails/*.md`.

## O que é
Um motor de **regras declarativas** que decide sobre **factos tipados** (`FactKind::{Tool,
Path}`) com veredicto (`Allow`/`Deny`/`Escalate`) e evidência — em vez de lógica dispersa
no código. No katu: `katu-policy` puro (`facts`, `rule`, `glob`, `paths`, `argv`,
`approval`, `decision`), com `containment.toml` como dado versionado (deny_read,
deny_sensitive_read, deny_write_outside) e **análise de `argv`** (não regex).

## O que o ktu tem hoje
- `StructuralGuard` (capacidade do `AgentSpec` + confinamento de path + caps) — I10.
- **Guardrails declarativos** em `.agents/guardrails/*.md` (`ChainGuard`, fail-closed) — I16.
- Mitigação de `bash` por análise de `argv` determinística.

Ou seja: o **seam** (`Guard::evaluate` sobre factos) já é o mesmo tipo de factos do katu;
falta o motor de regras genérico, a memória de aprovações e `Escalate` → aprovação.

## Porque não entrou
Os dois casos de uso actuais (capacidade estrutural + guardrails declarativos) ficam
cobertos sem um motor genérico. Um motor de política completo sem consumidor é
exactamente o erro do katu (`plan.md` §0).

## Pré-condições para reconsiderar
- Uma **regra real** que não caiba no schema declarativo actual (ex.: decisão por
  combinação de factos, ou aprovação memorizada por caminho).
- Um **`Escalate`** que precise de aprovação humana interactiva na porta.

## Custo e risco
- **Esforço:** médio-alto (motor + schema + testes adversariais).
- **Risco de não fazer:** regras cada vez mais complexas em `.md`/código.
- **Risco de fazer (YAGNI):** reintroduzir a complexidade que afundou o katu.

## Decisão pendente
Promover só quando surgir a 1.ª regra que o schema actual não exprima; nesse momento,
migrar `.agents/guardrails/*.md` para o motor (sem perder o fail-closed).
