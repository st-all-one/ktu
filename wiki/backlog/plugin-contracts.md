# Chord (contratos + estado replicado de plugins)

- **Estado:** candidata (do `pi`) — **avaliar** (avançado).
- **Origem:** `REF/pi/packages/chord` (11 k linhas); `wiki/revisao-pi.md` §2.11.
- **Âncora:** `crates/ktu-plugin/`, `crates/ktu-core/src/plugin/`.

## O que é
Um **host de plugins tipado** com **contratos** (verificação de interface), **estado
replicado** e **RPC** sobre *links* JSON.

## Porque interessa
O ktu tem plugins por **processo** + hot reload por **geração** (I11/I12). O *chord*
acrescenta **contratos** (o plugin declara a interface que cumpre) e **estado replicado**
(entre host e plugin). É mais avançado do que o ktu precisa hoje.

## O que o ktu tem hoje
Manifesto (`plugin.toml`: entrypoint/events/mode/timeout/daemon), JSON-lines sobre stdio,
timeout, gate fail-closed, hot reload por geração.

## Pré-condições para reconsiderar
- Plugins complexos que precisem de **contrato** versionado e **estado partilhado**.
- Um caso de uso que justifique RPC tipado.

## Custo e risco
- **Esforço:** alto (contratos + estado replicado + RPC tipado).
- **Risco de não fazer:** baixo (os plugins actuais servem).
- **Risco de fazer (YAGNI):** subsistema avançado sem consumidor.

## Decisão pendente
Manter fora. Se entrar, começar pelos **contratos** (verificação de interface no
manifesto), que é a parte mais simples e útil; o estado replicado fica para depois.
