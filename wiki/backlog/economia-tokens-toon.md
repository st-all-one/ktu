# Economia de tokens: TOON/colunar + read views + ToolReport

- **Estado:** deferido.
- **Origem:** katu ADR `0005-formato-colunar-d39.md`, `0006-toon-colunar-v3.md`,
  `0007-projecoes-model-facing.md`; `plan.md` §2.12-B.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs`, `crates/ktu-core/src/taint.rs`.

## O que é
Reduzir tokens no contexto do modelo:
- **Formato TOON/colunar** para tool results (mais denso que JSON).
- **Views de leitura** (`outline`/`summary`/`symbol`/`diff`/`full`) em vez do ficheiro inteiro.
- **`ToolReport`** como **ponteiro + delta truncado**, não o corpo inteiro.

## O que o ktu tem hoje
- Tool results em **texto simples**, truncados por caps; envelope *taint* (I13).
- O gate passou com equivalência de grau **sem** TOON.

## Porque não entrou
A economia de tokens é uma optimização; o gate já passou. Sem sintoma (contexto a
estourar / custo a subir), é complexidade prematura. Nota: no katu, a mudança de
formato foi **fonte de bugs** de wire.

## Pré-condições para reconsiderar
- Sintoma medido: contexto/limite de tokens a apertar, ou custo remoto relevante.
- Um benefício medido (benchmark) que justifique o formato.

## Custo e risco
- **Esforço:** médio-alto (formato + views + testes de contrato de bytes).
- **Risco de não fazer:** uso ineficiente de contexto em sessões longas.
- **Risco de fazer (YAGNI):** formato novo a introduzir bugs de wire (lição do katu).

## Decisão pendente
Adiar até haver número. Se entrar, fazê-lo **por trás de uma opção** e com goldens de
bytes, medindo o ganho antes de o tornar por omissão.
