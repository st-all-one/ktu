# 0010 — A CLI é o front primário

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-cli/src/`, `crates/ktu-port/src/`

## Contexto

O katu investiu numa TUI e num *daemon* antes de o loop funcionar. O ktu precisava de decidir onde
gastar o esforço de front-end — e onde não gastar.

## Decisão

O front **primário** é a CLI (`ktu run`, `ktu session`, subcomandos de conhecimento). A TUI
(`ktu tui`) e o servidor HTTP (`ktu serve`) são **clientes da mesma porta única** e não têm lógica
de agente: nenhum deles decide permissões, compõe contexto ou conhece invariantes.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| TUI-first | foi parte do erro do katu: esforço de apresentação antes de o loop ser estável |
| Biblioteca-first (sem CLI utilizável) | o utilizador não consegue operar o agente; a CLI é também o veículo do protocolo (`ktu prime`) |
| `daemon` sempre ligado | acrescenta ciclo de vida e IPC sem consumidor |
| Lógica de sessão no front-end | duplicaria o loop e quebraria I3 |

## Consequências

- O REPL **não tem lógica própria** (M4): comandos `/` viram comandos da porta; a fachada decide.
- Um front novo custa ~1 adaptador (a porta), não um segundo agente.
- Custo: a CLI cresce em subcomandos — daí o teto de superfície
  ([0011](0011-limite-de-400-linhas-e-firewall-mecanico.md)).
