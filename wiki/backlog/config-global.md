# Configuração global — `~/.config/ktu/`

- **Estado:** **implementado** (H2, parcial: recursos globais + migrações adiados) — feito.
- **Origem:** [CLI e configuração](../cli-e-config.md) §3; modelo: `kd` (dois níveis).
- **Âncora:** `crates/ktu-cli/src/init.rs`, `crates/ktu-core/src/layout.rs`.

## O que é
A **config do utilizador** (global), fora do projecto, com defaults e segredos:

```
~/.config/ktu/
  auth.toml        # credenciais de providers/modelos — 0600, NUNCA no projeto, nunca logado
  config.toml      # defaults de comportamento (provider, orçamento, effort, max_steps)
  memory.toml      # vista do ktu sobre a config da memória (projeta no knudge; uma só verdade)
  skills/          # skills globais (sobrescritas por locais de mesmo nome)
  guardrails/      # guardrails globais (PISO: sempre ativos; locais só adicionam)
  plugins/         # plugins globais (aditivos)
  recipes/         # receitas globais (se implementadas — G1)
```

O **local** (`.agents/`) mantém o que é do projecto: memória (dados), personalização, e as
skills/guardrails/plugins/recipes locais (sobreposição por nome).

## Porque interessa
É o que falta para o uso #4: personalização **por utilizador** (auth, defaults, piso de guardrails)
sem violar M6 (o **projecto** é a autoridade). Espelha o modelo do `kd` (global + projecto, projecto
ganha chave a chave).

## O que o ktu tem hoje
Só defaults no código + env + flags; **sem ficheiro de config** (nem global nem local).

## Pré-condições para reconsiderar
- Nenhuma — é o eixo da configuração.

## Custo e risco
- **Esforço:** médio (dois níveis + precedência + `ktu config get/set/list`).
- **Risco de não fazer:** sem personalização; segredos só por env.
- **Riscos a mitigar:**
  1. **`auth.toml` vazar** — 0600, fora do repo, **nunca** no log/contexto; teste "auth não ecoa".
  2. **Dupla verdade da memória** — `memory.toml` é **vista** que projeta no knudge, não um motor
     paralelo.
  3. **Guardrail global removível** — não é: é **piso** (aditivo); tentar desligar = ignorado.

## Decisão pendente
Incorporar. **Assimetria declarada:** skills/plugins/recipes globais = *defaults* (sobreponíveis);
guardrails globais = *piso* (não removíveis). Precedência: comportamento `flag>params>local>global`;
credenciais `flag>env>auth.toml`.
