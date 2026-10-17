---
id: deny-dotenv
version: 1
applies_to: '*'
fact: path
match: '*.env'
decision: deny
reason: segredos
---

# Porquê

Impede a leitura de ficheiros de segredos (`.env`, `.env.local`, ...).
