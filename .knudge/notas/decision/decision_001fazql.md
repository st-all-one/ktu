---
id: decision_001fazql
type: decision
statement: ktu doctor diagnostica o projecto e aponta as outras portas; o diagnostico da memoria tem porta propria
created_at: 2026-10-18T17:35:43.359Z
body_hash: 3a07b6a8
schema_version: 2
revision: 2
anchors: [crates/ktu-cli/src/doctor_cmd.rs, crates/ktu-cli/src/memory_cmd.rs]
---
Decisao (diagnostico, pos-F6). (1) O `ktu doctor` passa a ser o diagnostico do **projecto** (layout, AGENTS.md, skill) e **aponta** `ktu config doctor`/`ktu memory doctor`/`ktu providers doctor` em vez de agregar tudo: cada area tem uma porta, e o doctor nao duplica a fonte (o que estava era pior - prometia layout/conhecimento e corria o diagnostico do provider). (2) O diagnostico da memoria passa a `ktu memory doctor` e o `--memory` sai: uma so porta por area (o `--memory` era uma segunda porta escondida na area errada). (3) O marcador de versao do bloco de protocolo passa a ser **lido** (git::agents::version_in): bloco velho => "corre `ktu init`", bloco mais recente que o binario => "actualiza o ktu" - a politica espelha a do schema de config (fail-closed, nunca rebaixar um ficheiro que nao se sabe ler). (4) Cada item do diagnostico leva o **remedio** (a licao do remedio por regra do F4): "FALTA" sem accao nao ajuda ninguem. Alternativas descartadas: (a) um `ktu doctor` que corre tudo (layout + config + memoria + provider) - uma chamada a fazer trabalho de quatro, com o exit code a misturar saude de coisas diferentes; (b) manter `doctor --memory` e acrescentar `memory doctor` - duas portas para o mesmo; (c) inferir a versao do bloco pelo conteudo (sem o marcador) - heuristica fragil, e o marcador existe exactamente para isto.