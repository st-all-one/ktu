# Custom distros / deeplinks

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/CUSTOM_DISTROS.md`, `recipe_deeplink.rs`; `wiki/revisao-goose.md` §3.
- **Âncora:** `install.sh`, `scripts/dist.sh`.

## O que é
Construir **distribuições personalizadas** do agente (providers/extensões/branding
pré-configurados) e **deeplinks** para lançar receitas a partir de um URL.

## Porque não entra
O ktu distribui um binário simples (`install.sh`/releases). Distros personalizadas e
deeplinks são uma camada de produto sem consumidor.

## Pré-condições para reconsiderar
- Um caso de uso de "kit" pré-configurado para terceiros.

## Custo e risco
- **Esforço:** médio.
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** camada de produto antes de tempo.

## Decisão pendente
Manter fora. As **receitas** (backlog 19) cobrem o "workflow reutilizável" sem a camada de
distribuição.
