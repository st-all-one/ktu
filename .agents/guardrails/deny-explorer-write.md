---
id: deny-explorer-write
version: 1
applies_to: explorer
fact: tool
match: write_file
decision: deny
reason: capability
---

# Porquê

O `explorer` é read-only por construção; esta regra é defesa em profundidade.
