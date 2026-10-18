# Nome automático de sessão

- **Estado:** **incorporada** (H10) — título heurístico determinístico (≤ 8 palavras), guardado no
  *checkpoint* e mostrado por `ktu session ls`; *fallback* ao `id`.
- **Origem:** `REF/goose/crates/goose/src/session/session_naming.rs`;
  `wiki/revisao-goose-profunda.md` §1.6.
- **Âncora:** `crates/ktu-cli/src/session_store.rs`.

## O que é
Após `MSG_COUNT_FOR_SESSION_NAME_GENERATION = 3` mensagens, gera um **título curto** para a
sessão (limpa tags XML, extrai um título de ≤ 8 palavras).

## Porque interessa
Listar sessões por `id` é pouco legível; um nome curto gerado do conteúdo melhora a
usabilidade da TUI/`serve`. Simples; pode ser heurístico (sem LLM) e determinístico.

## O que o ktu tem hoje
Sessões identificadas por id (`session_store.rs`).

## Pré-condições para reconsiderar
- Listagem de sessões na TUI/CLI.

## Custo e risco
- **Esforço:** baixo (heurística + campo `title`).
- **Risco de não fazer:** sessões ilegíveis.
- **Risco de fazer:** nome mau — heurística determinística + fallback ao id.

## Decisão pendente
Promover. Título heurístico determinístico (≤ 8 palavras, sem LLM) guardado na sessão;
fallback ao id.
