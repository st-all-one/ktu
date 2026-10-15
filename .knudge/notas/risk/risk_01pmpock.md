---
id: risk_01pmpock
type: risk
statement: O confinamento de path (resolve_within) e lexical e nao deteta symlinks que escapam do root
created_at: 2026-10-13T01:40:13.871Z
body_hash: fc8c4879
schema_version: 2
anchors: [crates/ktu-core/src/guard.rs]
---
Evidência: guard.rs::resolve_within recusa caminhos absolutos e .. que escape, mas nao canonicaliza; um symlink dentro do root a apontar para fora passa. Consequência: aceitavel em F1 (so leitura); antes das tools de escrita (G2: write/edit) canonicalizar o path real e verificar o prefixo do root.