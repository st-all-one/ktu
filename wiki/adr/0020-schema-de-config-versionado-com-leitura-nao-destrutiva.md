# 0020 — Schema de config versionado com migração de leitura não-destrutiva

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/config/migrate.rs`, `crates/ktu-cli/src/config_doctor.rs`,
  `wiki/implementation_phase3/F5-migracoes-config.md`

## Contexto

A config em dois níveis (H2) é um contrato **em ficheiro**, escrito à mão pelo utilizador e por
`ktu init`: `.agents/config.toml` e `~/.config/ktu/config.toml`. Até aqui não declarava **versão**
nenhuma, pelo que uma mudança de schema seria indistinguível de um erro de escrita — e o ktu não
teria como saber se sabe ler o que encontrou. O item F5 ([dossiê §2](../pendencias.md), **D2**)
pedia versão de schema e migração idempotente, com uma variante de **leitura não destrutiva**
(lição do `goose`: um caminho de leitura em memória não pode remover chaves planas que outros
chamadores ainda consultam).

**Gatilho declarado vs. trabalho feito (honestidade):** o gatilho do F5 era «o schema mudar para
**v2** com configs v1 em uso» — **não disparou**: não há nenhuma chave obsoleta (as `KEYS` nunca
mudaram, verificado no histórico) e nenhuma config antiga em uso. O que se implementou é o
**mecanismo** (versão + migração), que é pré-condição de qualquer migração futura: sem versão não
há base para migrar. O plano (`Fn-*.md`, passos 1–5) era explícito e foi seguido, com **um desvio
declarado**: o passo 2 previa que a variante de escrita pudesse **remover** chaves planas antigas;
como não existe nenhuma, a lista está vazia e o mecanismo de retirada é exercitado por um **passo
de ensaio** nos testes (não se inventou uma chave obsoleta só para a ter).

## Decisão

1. **Versão declarada no ficheiro** (`schema = 1`); a **ausência** significa **v0** (o formato
   anterior). Escrever **carimba** sempre a versão actual — um ficheiro escrito pelo ktu sai
   versionado.
2. **Duas variantes de migração, de propósito:**
   - `read_migrate` (**leitura**): devolve o documento na versão actual **em memória**,
     acrescentando o que falta e normalizando valores antigos, mas **nunca removendo chaves**;
   - `migrate` (**escrita**): o documento a persistir; **pode** remover as chaves que um passo
     declare obsoletas e devolve se mudou algo (idempotência).
3. **Fail-closed em versão futura**: `schema = 3` num binário v1 é **erro legível** e **nenhuma**
   escrita — nunca se «migra para trás» sobre um ficheiro que não se sabe ler.
4. **Migração nunca é silenciosa**: quem escreve avisa em `stderr` quando o ficheiro mudou.
5. **Diagnóstico separado da migração**: `ktu config doctor` mostra a versão de cada nível e se
   migraria, **sem escrever**; quem migra é a escrita (`config set`/`unset`).
6. O **v1** concreto normaliza `deferred_tools` (a forma plana/CSV, que a CLI `set` aceita e um
   ficheiro escrito à mão pode ter) para lista — a forma que o schema tipado exige.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Uma só função de migração (sem variante de leitura) | a leitura passaria a poder remover: é o defeito que o `goose` pagou e que o plano queria evitar por construção |
| Ler tolerante a chaves desconhecidas (`deny_unknown_fields` desligado na leitura) | abriria mão da decisão do H2: uma gralha numa chave deixaria de ser erro e passaria a ser ignorada em silêncio |
| Migrar automaticamente na leitura (reescrever o ficheiro ao ler) | uma leitura não deve escrever: `ktu config get` a alterar o ficheiro surpreende e quebra o hábito de inspeccionar antes de mexer |
| Não versionar (migrar «pelo formato», detectando a forma antiga) | heurística frágil: sem versão declarada, o v3 é indistinguível de um erro de escrita |
| Inventar uma chave obsoleta para exercitar a retirada | seria uma migração fictícia; a lista fica vazia e o mecanismo é testado com um passo de ensaio |
| Um `ktu config migrate` dedicado (comando novo) | superfície a mais: a escrita já migra, e o `doctor` diz o que aconteceria sem tocar em nada |

## Consequências

- Uma config v0 **continua a ser lida** (normalizada em memória) e passa a v1 na primeira escrita,
  com aviso — sem `ktu init`, sem passo manual e sem perder chaves.
- O contrato está **travado**: `xtask gates` G20 prova v0→v1, idempotência (hash igual em duas
  passagens), leitura não destrutiva (o ficheiro fica byte-idêntico) e fail-closed (v3 ⇒ erro
  legível + ficheiro intacto); o núcleo tem testes para cada um dos caminhos.
- **Contrapartida assumida:** o `migrate` pode retirar chaves, portanto a leitura de um ficheiro
  que as tenha exige migração (escrita) antes — o `doctor` é o que o diz. Hoje a lista está vazia,
  pelo que o caso não existe; fica montado para o v2.
- O v1 muda o **tipo** de `deferred_tools` (string → lista). Quem tiver a forma antiga não perde
  nada: a leitura normaliza e a escrita normaliza e grava.
