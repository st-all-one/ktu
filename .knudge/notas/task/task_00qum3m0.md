---
id: task_00qum3m0
type: task
statement: Diagnostico: ktu memory doctor com porta propria e ktu doctor do projecto (le a versao do bloco no AGENTS.md)
created_at: 2026-10-18T17:35:22.090Z
body_hash: 7177457f
schema_version: 2
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-18T17:35:40.335Z
    notes: "Feito: ktu memory doctor (porta propria) + ktu doctor do projecto com o marcador de versao do AGENTS.md; G14 15/15, make check 0, 446 testes, 12 gates / 93 verificacoes verdes."
anchors: [crates/ktu-cli/src/doctor_cmd.rs, crates/ktu-cli/src/memory_cmd.rs, crates/ktu-cli/tests/doctor.rs, xtask/src/gates/project.rs]
status: closed
scope: task
---
Duas lacunas reais encontradas ao fechar o F6. (1) `ktu memory doctor`: o diagnostico da memoria existia mas escondido atras de `ktu doctor --memory`; passa a porta propria na area da memoria (MemoryAction::Doctor) e o `--memory` sai (uma so porta por area). (2) `ktu doctor` diagnosticava o *provider* apesar de o --help prometer "layout e conhecimento": passa a diagnosticar o **projecto** (novo crates/ktu-cli/src/doctor_cmd.rs) - layout (INIT_DIRS), `AGENTS.md` (marcador de versao lido por git::agents::version_in, que existia documentado e testado e **ninguem chamava**) e skill - cada item com **remedio**, e o comando **aponta** as outras portas (ktu config doctor, ktu memory doctor, ktu providers doctor) em vez de as duplicar. Bloco velho => "corre `ktu init`"; bloco mais recente que o binario => "actualiza o ktu" (nunca rebaixa). Exit 0 se tudo em dia, 1 se falta algo. Prova: 3 unit em doctor_cmd.rs, 4 de integracao em crates/ktu-cli/tests/doctor.rs, tests/memory_cli.rs atualizado para a porta nova, e o gate G14 estendido de 12 para 15 verificacoes (doctor em dia depois do init, bloco velho detectado com remedio, memory doctor com porta propria). surface.toml: subcomandos 50->51, flags 89 (entra o --root do memory doctor, sai o --memory).

Duas lacunas resolvidas (task_00qum3m0). (1) `ktu memory doctor`: o diagnostico da memoria (notes, malformed, bytes, broken_anchors, canal semantico) existia mas so atras de `ktu doctor --memory`; passou a `MemoryAction::Doctor { root }` na area da memoria e o `--memory` saiu (uma so porta por area; envelope passa a "memory doctor"). (2) `ktu doctor`: corria o diagnostico do *provider* (providers::doctor com base_url fixo e Probe::Skip) apesar de o --help prometer "Diagnostica o layout e o conhecimento do projecto"; agora e o novo crates/ktu-cli/src/doctor_cmd.rs - tres itens com remedio: layout (INIT_DIRS presentes), agents_md (git::agents::version_in vs git::agents::VERSION: ausente/sem bloco => "corre `ktu init`"; v<VERSION => "corre `ktu init`"; v>VERSION => "actualiza o ktu (`ktu self upgrade`)" - nunca rebaixa) e skill (.agents/skills/ktu/SKILL.md). Cada item tem State::Ok|Pending (enum, nao bool) + remedy (uma recusa sem remedio nao e accionavel) e o comando aponta as outras portas (ktu config doctor, ktu memory doctor, ktu providers doctor) em vez de as duplicar; exit 0 se tudo em dia, 1 se falta algo. O `version_in` existia publico, documentado e testado e **ninguem o chamava** - o marcador era escrito pelo init e nunca lido. Prova: 3 unit em doctor_cmd.rs, 4 de integracao em crates/ktu-cli/tests/doctor.rs (projecto vazio aponta o init; depois do init healthy; bloco velho e bloco mais recente com remedios diferentes; memory doctor com porta propria), tests/memory_cli.rs atualizado, gate G14 de 12 para 15 verificacoes. surface.toml: subcomandos 50->51 e flags 89 (entra --root do memory doctor, sai --memory).