# Adaptador GDK (goose-sdk)

- **Estado:** **decidido: não implementar** (G9; `decision_00aw5etj`).
- **Origem:** `plan.md` §2.5/§2.12-B; `task_010qrmko`; `decision_00aw5etj`.
- **Âncora:** `crates/ktu-provider/src/wire.rs`.

## O que é
Um crate `ktu-provider-gdk` atrás da feature `gdk`, usando o **goose Development Kit**
para alcançar providers além do llama.cpp (Anthropic, OpenAI, opencode-zen, …) sem
escrever dialetos próprios.

## O que o ktu tem hoje
- Provider **built-in** `llama.cpp` (llama-server, SSE, tool-calling nativo).
- Provider **OpenAI-compatible fino** (`opencode-go`), suficiente para os providers
  remotos usados.
- Um `Flavor` de wire (`OpenAi`/`Llama`) já preparado.

## Porque foi decidido não implementar
Avaliação com evidência (`decision_00aw5etj`):
1. `goose-sdk` (`0.1.0-alpha.12`) com features por omissão só reexporta **tipos ACP**
   (fala com `goose acp` por stdio); a biblioteca de providers é `goose-providers` (alpha).
2. `goose-providers` puxa **~167 pacotes extra** (Cargo.lock de crate novo), incluindo
   **`rmcp`/MCP** (na lista YAGNI §0); o ktu tem 267 crates únicos.
3. Exige **rede** (sem test double próprio) → não-determinístico de testar.
4. **Desajuste arquitectural:** o goose é um **agente** (ACP) ou traz tipos de mensagem
   próprios; integrá-lo como `Provider` obriga a adaptador e/ou a um segundo agente.

## Pré-condições para reconsiderar
- Necessidade de **muitos** dialetos em simultâneo (≥3) que justifique a dependência.
- Um `goose-sdk` estável (não-alpha) com API de provider nativa em Rust.

## Custo e risco
- **Esforço:** alto (adapter + dep tree grande + MSRV/alpha).
- **Risco de não fazer:** precisar de um dialeto novo → usar `provider-dialetos.md`.
- **Risco de fazer:** contrariar simplicidade/determinismo/YAGNI.

## Decisão pendente
Fechada. Reabre só se as pré-condições acima se verificarem; até lá, o caminho é
[providers finos por dialeto](./provider-dialetos.md).
