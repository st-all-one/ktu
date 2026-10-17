# Roaming agents / peerbook

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose-roaming`; `documentation/docs/guides/roaming-agents.md`;
  `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-port/`.

## O que é
Agentes que **descobrem e comunicam** com outros agentes numa rede (peerbook, handshake,
relay) — coordenação multi-agente distribuída.

## Porque não entra
O ktu tem **um** agente + subagentes read-only locais (fan-out). Coordenação distribuída é
um subsistema (descoberta, identidade, transporte, segurança) sem consumidor.

## Pré-condições para reconsiderar
- Um caso de uso real de múltiplos ktu em máquinas diferentes a colaborar.

## Custo e risco
- **Esforço:** alto (identidade + transporte + segurança).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** superfície de rede e segurança sem necessidade.

## Decisão pendente
Manter fora. O fan-out local cobre a coordenação actual.
