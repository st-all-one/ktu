---
id: deny-secret-files
version: 1
applies_to: '*'
fact: path
match: '*.pem'
decision: deny
reason: segredos
---

# Porquê

Chaves e certificados nunca entram no contexto do modelo.
