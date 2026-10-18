---
id: fact_00lz1oai
type: fact
statement: Instrucoes locais: o AGENTS.md mais proximo e anexado ao resultado da leitura de um ficheiro
created_at: 2026-10-18T03:20:36.915Z
body_hash: a1c09855
schema_version: 2
anchors: [crates/ktu-core/src/ports/instructions.rs, crates/ktu-core/src/agent/delegation.rs, crates/ktu-cli/src/instructions.rs, wiki/convencoes.md]
---
A11 (H10, implementado a pedido do dono antes da H11). Nova porta ktu-core::ports::Instructions (nearest(path) -> Option<Local{source,text}>) com o fake MapInstructions (o prefixo de caminho mais longo ganha, deterministico) e Capabilities.instructions + KtuBuilder::instructions. O loop anexa as instrucoes ao RESULTADO da tool (so read_file/edit_file e so em sucesso), uma vez por turno e por FONTE (BTreeSet de fontes ja injectadas), via Driver::with_local_instructions (movido para delegation.rs por causa do limite de 400 linhas), emitindo context.local pela porta Logger: o texto fica no log, logo model-visible <=> logged. agents_md::label(root, path) passou a publico (rotulo relativo com /, que e a chave de deduplicacao) e o adaptador da CLI (ktu-cli/src/instructions.rs, agents_md::nearest + condense sobre StdFs) esta ligado no turn_runtime e no host de subagentes. Formato do envelope: "\\n\\n[instrucoes locais: <fonte>]\\n<texto>". Testes: ktu-core/tests/loop.rs::local_instructions_are_attached_once_per_source (o mais proximo ganha; a 2a leitura em sub/ nao repete; a leitura fora de sub/ traz o da raiz; 2 eventos context.local) e ktu-cli/tests/commands.rs::reading_a_file_attaches_the_nearest_agents_md.