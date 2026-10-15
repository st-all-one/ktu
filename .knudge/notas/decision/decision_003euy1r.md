---
id: decision_003euy1r
type: decision
statement: O confinamento de path das tools de escrita resolve o caminho real (canonicalize) e verifica o prefixo do root
created_at: 2026-10-13T01:54:07.874Z
body_hash: 2bb2e3f2
schema_version: 2
anchors: [crates/ktu-tools/src/path.rs]
---
Evidência: ktu-tools/src/path.rs::resolve_existing/resolve_for_write canonicalizam o alvo (ou o directório pai, na escrita) e comparam com o root canonicalizado; resolve_within continua a barreira lexical. Consequência: symlinks dentro do root que apontem para fora são recusados; fecha o risk_01pmpock.