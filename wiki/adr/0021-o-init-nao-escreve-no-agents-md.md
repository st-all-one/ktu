# 0021 — O `ktu init` não escreve no `AGENTS.md`

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-cli/src/init.rs`, `crates/ktu-core/src/git/agents.rs` (removido),
  `crates/ktu-core/src/prompt.rs` (`BASE`), `crates/ktu-cli/src/prime.rs`

## Contexto

O `ktu init` (H1) escrevia um **bloco gerido** no `AGENTS.md` do projecto
(`<!-- ktu:start -->` … `<!-- ktu:end -->`, com marcador de versão), um *router* que dizia:
«`.agents/` é conhecimento, `.ktu/` é runtime; ciclo: entender → planear → editar → correr →
verificar; consulta a memória (`memory_ask`) e grava (`memory_write`); protocolo completo em
`ktu prime`». O objectivo era que um agente que abrisse o projecto soubesse o protocolo.

Verificado o que o ktu faz com esse ficheiro, o bloco é **redundante e custa contexto**:

- o `AGENTS.md` é uma **camada do system prompt** (`agents_md::discover` → `prompt::compose_layers`)
  — entra em **todos** os turnos, sempre;
- o `prompt::BASE` já diz a mesma disciplina de memória («consulta `memory_ask` antes de decidir e
  grava cada aprendizado com `memory_write`») e o `ktu prime` já tem o **CICLO** e as **CONVENÇÕES**
  — o bloco era um **subconjunto** dos dois;
- o protocolo do ktu é **do ktu**: o agente que lê `AGENTS.md` é o próprio ktu, que já traz a base
  (ou a directriz do projecto em `.agents/SYSTEM.md`) e pode pedir o protocolo com `ktu prime`;
- o `AGENTS.md` é o ficheiro de instruções **do projecto** — escrito pelo humano, para as regras
  daquele projecto — não um sítio onde a ferramenta se anuncia.

E a alternativa «serve para outros agentes saberem que o projecto usa ktu» já tinha sido avaliada e
fechada: as **receitas de cliente** ([F6](../implementation_phase3/F6-self-setup.md)) não têm
consumidor, e o ktu não tem servidor MCP.

## Decisão

**O `ktu init` não escreve no `AGENTS.md`** — nem o cria, nem o altera. O ficheiro é do projecto.

- O módulo `ktu_core::git::agents` (bloco, marcadores, `protocol_block`, `version_in`) **sai**: sem
  bloco gerado, o marcador de versão não tem nada a ler.
- O protocolo do agente vive onde é **vivo**: `prompt::BASE` (identidade e disciplina), o
  `ktu prime` (protocolo completo, impresso pelo binário — nunca uma cópia que apodrece) e, por
  projecto, `.agents/SYSTEM.md` / `.agents/APPEND_SYSTEM.md` (directriz do humano).
- O `ktu doctor` deixa de diagnosticar o bloco: passa a verificar **layout e skill** (o que o
  `ktu init` realmente gera), mantendo o remédio por item.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Manter o bloco no `AGENTS.md` (o que estava) | Duplica `BASE` + `prime` dentro do **system prompt de cada turno**: custo permanente, informação nenhuma |
| Mover o bloco para `.agents/SYSTEM.md` (o ficheiro do agente) | Seria escrever na **directriz do humano** (que *substitui* a base) para repetir conteúdo que o `BASE` e o `prime` já têm — e o ktu passaria a gerar um prompt de sistema |
| Manter o bloco mas excluí-lo do prompt | O `AGENTS.md` é lido **por desenho** (instruções do projecto, aninhadas e injectadas na leitura de ficheiros); filtrar blocos por marcador seria complexidade para preservar uma cópia inútil |
| Um `ktu doctor` que só avisa quando o bloco existe e está velho | Código para um caso que deixa de existir (nenhum projecto novo tem bloco) — e um aviso que nunca dispara é ruído |
| Manter `version_in` «para o futuro» | Sem consumidor: o protocolo é servido pelo binário (`ktu prime`), pelo que não há versão a sincronizar em ficheiro |
| Deixar o `ktu init` **remover** um bloco antigo | O ktu não escreve no `AGENTS.md`; projectos initados por versões anteriores guardam o bloco até o humano o apagar (pré-1.0, sem migração) |

## Consequências

- O `AGENTS.md` do projecto fica **inteiramente nas mãos do humano** — o ktu só o lê. O teste
  `the_project_agents_file_is_never_touched` e o gate G14 («o init nao toca no AGENTS.md do
  projecto») travam-no.
- O system prompt de cada turno perde uma camada redundante (~120 tokens de *router* por turno) e
  o protocolo passa a ter **uma só fonte**: o binário.
- **Correcção de uma decisão anterior:** o fecho do **F6** usou, entre outros, o argumento «o
  `ktu init` já escreve o bloco no `AGENTS.md`, que os clientes lêem». Esse argumento **cai** com
  esta ADR — o fecho mantém-se pelos restantes (não há MCP no ktu, não há receita de cliente com
  consumidor, o gatilho não disparou), e o gatilho de **D3** passa a ser «um cliente concreto que
  precise de configuração própria», sem contar com o `AGENTS.md`.
- `ktu prime` continua a ser o protocolo canónico (e é o que os testes byte-a-byte do `prime`
  protegem); as convenções do projecto que o humano queira fixar vão para `.agents/SYSTEM.md` ou
  `.agents/APPEND_SYSTEM.md`.
