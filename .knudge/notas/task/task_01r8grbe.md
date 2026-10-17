---
id: task_01r8grbe
type: task
statement: install.sh + packaging + CI/MSRV + releases
created_at: 2026-10-13T00:51:17.804Z
body_hash: 65a57812
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T11:24:37.739Z
    notes: "install.sh (fonte + release com verificacao SHA-256 + KTU_RELEASE_BASE), scripts/dist.sh (tarball+sha256 deterministico por alvo/versao), make dist, .github/workflows/release.yml (tag v* -> matriz linux x86_64 + macos aarch64 -> anexa a release), ci.yml jobs msrv (cargo check 1.99) e dist. Evidencia local: make dist produziu ktu-0.0.1-x86_64-unknown-linux-gnu.tar.gz + sha256; install.sh validado em 4 caminhos (fonte ok, release file:// checksum SUCESSO, checksum errado falha, repo inexistente mensagem limpa). make check e make supply-chain verdes."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00fgs8ld -->