---
question_id: Q4
question_issue: "Com base em /exploration/knudge/[agent]/knudge.md, crie um aquivo EPIC.md ao lado dele, descrevendo como reimplementar em typescript, vantagens e desvantagens, contneod no mínomo um plano mestre e diversas issues, encadeads sequencialmente"
expected_answer_ideia: "Deve ser capaz de construir a cadeia de tarefas, vantagens, desvantagens globais e a cadeia de tarefas deles"
provider: opencode-go
model: longcat-2.5-preview-free
thinking-level: xhigh
cache: true
same_session: true
compacted: true
persistence_probe: true
agent: ktu
---
Criei `exploration/knudge/ktu/EPIC.md` ao lado do resumo. O ficheiro inclui:

- **Contexto e motivação** da migração Rust → TypeScript
- **Tabela de vantagens e desvantagens** com mitigações para cada risco
- **Arquitetura alvo** em monorepo NPM com packages `@knudge/core`, `@knudge/cli`, `@knudge/mcp`, `@knudge/fs`
- **Plano mestre em 6 fases** (Fundação → Core → Retrieval → CLI → MCP/Embeddings → Qualidade/Migração), cada uma com issues e tasks
- **Issues encadeadas sequencialmente** (EPIC-000 até EPIC-500) com dependências explícitas
- **Convenções e limites** (arquivos ≤ 300 linhas, sem `any` injustificado, erros como valores, determinismo via port `Rng`)
- **Riscos conhecidos** e critérios de sucesso globais
- **Referências** ao resumo, decisões Dxx, TOON, e bibliotecas TypeScript

A estrutura foi pensada para ser diretamente conversível em issues de GitHub/GitLab com parentesco claro entre épico → fase → issue → task.
