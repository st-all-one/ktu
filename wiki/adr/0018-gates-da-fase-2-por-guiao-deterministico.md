# 0018 — Gates da Fase 2 por guião determinístico (provider `scripted`)

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-cli/src/provider_script.rs`, `xtask/src/gates/`, `wiki/gates/`

## Contexto

Os gates da Fase 2 (G10–G15) exigem **prova**, não afirmação: cada um tem de correr o agente numa
cadeia real (≥15 turnos numa sessão, três caminhos com o mesmo log, concorrência, memória) e
arquivar artefactos. O modelo de referência (Ornith-1.5-9B Q4_K_M, CPU-only) mediu **~8 min por
completion** neste host — uma cadeia de 15 turnos seria inviável, e uma corrida ruidosa não é
repetível (o critério exige «o mesmo gate, duas vezes ⇒ o mesmo resultado observável»).

## Decisão

Os gates correm com **providers determinísticos**: `mock` (eco) para os caminhos triviais e um
novo **`--provider scripted --script <ficheiro.jsonl>`** (uma `Completion` por linha) para as
cadeias longas. O guião é um **fluxo do processo** (cada passo do modelo consome a linha seguinte,
mesmo entre turnos), o que permite `--batch --sequential` e o REPL percorrerem uma tarefa real sem
inferência. `xtask gates` corre o binário de ponta a ponta, verifica os critérios e arquiva
`wiki/gates/{summary.json,artifacts/<gate>/}`; `make gates` é o alvo (fora do `make check`, porque
escreve evidência versionada; a CI corre-o como passo próprio).

O modelo real fica como **observação opcional** (`--real-model`), nunca como critério que não se
consegue cumprir.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Correr tudo com o modelo de referência | ~8 min por completion neste host; a corrida não cabe no orçamento e não é repetível |
| Só `--provider mock` (eco) | não executa uma tarefa (não escreve, não corre testes): o artefacto do G10 não seria verificável |
| Um *fake provider* só nos testes do núcleo | provaria a biblioteca, não o binário: os gates existem para provar o **produto** de ponta a ponta |
| Gravar uma sessão real e fazer *replay* (`--record`) | mais superfície e um artefacto binário grande para o que o guião (revisto como código) já dá — fica no dossiê se aparecer sintoma |
| Guião por turno (um ficheiro por turno) | obrigaria a plumbar um índice de turno; o cursor do processo é uma linha de estado e serve `--batch --sequential` e o REPL |

## Consequências

- Os gates são **determinísticos e repetíveis**: duas corridas dão o mesmo quadro; o G12 prova a
  igualdade byte-a-byte do log entre os três caminhos e o determinismo do mesmo caminho.
- O guião é código revisto (fica em `wiki/gates/fixtures/`) e o artefacto do gate é o resultado
  daquele guião — não uma amostra de um modelo que muda de humor.
- **Contrapartida assumida:** o guião não mede **qualidade** de modelo. Essa barra continua a ser a
  bateria `p1`–`p4` (G4, com o modelo real) e a observação opcional do G11. Está registado como
  desvio em `wiki/gates/README.md` §4.
- O cursor do guião é **global ao processo**: em `--batch` paralelo a ordem deixa de ser
  determinística (documentado no `--help` e no módulo); use-se `--sequential`.
