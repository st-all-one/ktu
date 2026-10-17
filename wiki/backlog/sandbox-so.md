# Sandbox de SO (Landlock / seccomp)

- **Estado:** fora (YAGNI explícito).
- **Origem:** katu `18-jail-futuro.md`, `08-sandbox-fail-closed.md`; `plan.md` §0/§2.8.
- **Âncora:** `crates/ktu-tools/src/bash.rs`.

## O que é
Isolamento **real de SO** da execução de tools/`bash` (Landlock/seccomp/namespaces) — uma
fronteira de segurança, não apenas travas determinísticas.

## O que o ktu tem hoje
- `StructuralGuard` (capacidade + confinamento de path + caps) e guardrails fail-closed.
- **Mitigação determinística** do `bash`: análise de *argv* que recusa `~`/`$HOME`, `..`
  e caminhos absolutos fora do root (permite só prefixos de sistema read-only).
- Sem `unsafe` no núcleo; `bash` corre com as permissões do utilizador.

## Porque não entrou
A contenção do ktu é **soft/declarada** por decisão (§2.8): as travas determinísticas
cobrem o caso do agente, e o sandbox de SO é feature futura. Não há requisito de
multi-tenant nem de adversário real.

## Pré-condições para reconsiderar
- Requisito de segurança real (executar código não confiável, multi-tenant).
- Alvo de SO definido (Linux-only é aceitável?).

## Custo e risco
- **Esforço:** alto (por SO, testes, portabilidade).
- **Risco de não fazer:** o agente tem os poderes do utilizador (aceitável no escopo).
- **Risco de fazer (YAGNI):** complexidade de SO sem ameaça no escopo.

## Decisão pendente
Manter fora. A mitigação de `bash` cobre o risco observado (`risk_01ua5ei2`).
