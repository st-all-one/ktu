---
id: task_01ka615s
type: task
statement: Documentacao de integracao (wiki/integration) + crate de exemplo compilado ktu-embed
created_at: 2026-10-18T02:29:15.160Z
body_hash: b4c9ded2
schema_version: 2
tags: [fase2]
revision: 5
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:29:39.018Z
    notes: "Documentacao de integracao pedida pelo dono: wiki/integration/ com README (indice, quickstart, mapa de tarefas, contrato minimo de 10 pontos) e 10 documentos por topico - 01-panorama (onde vive a fachada, o que resolve vs injecta, threads T1/T2, o que NAO faz, quando nao usar), 02-construtor (referencia completa dos 16 setters com obrigatoriedade/defaults, o que o build() resolve, ownership/tempos de vida, 3 perfis de montagem), 03-turnos (o passo a passo interno, TurnOutcome, tabela de StopReason com o que fazer, cancelamento, modos I15, orcamento I14, guardrails/DenyReason, steering, hooks), 04-sessoes (o log E a sessao, Event e projecao de wire, persistencia pela porta Fs, retoma, checkpoint/compactacao, concorrencia/lock, projecao para o knudge), 05-portas-e-adaptadores (mapa das 11 portas com adaptador pronto vs escrever o teu, adaptadores prontos em detalhe, codigo copiavel para Clock/Logger/Steering/SubagentHost/Memory/Provider/ToolExecutor, tabela de fakes), 06-projecto (arvore .agents/.ktu, o que a fachada le no build, formato dos guardrails e permissoes, AGENTS.md, memory.toml/embeddings, config.toml e precedencia, quem cria os directórios), 07-erros-e-diagnostico (modelo de erros, fail-closed vs degradacao graciosa, catalogo, tabela sintoma->causa->correcao com 20 linhas, como investigar um turno, exit codes), 08-boas-praticas (ownership, runtime, orcamento/cancel, determinismo/testes, logs, seguranca, o que NAO fazer, config/versionamento, regras do projecto), 09-exemplos (10 receitas: teste com fakes, app real, servico por sessao, paralelismo sem static com std::thread::scope, memoria+embeddings, modo plano com aprovacao, plugins, subagentes, testes de configuracao, correr o exemplo), 10-para-ia (regras must/must not, mapa de simbolos para grep, checklists, anti-padroes->correcao, arvore de decisao, invariantes, contrato colavel num prompt)."
  - status: success
    recorded_at: 2026-10-18T02:29:39.189Z
    notes: "Crate de exemplo COMPILADO (membro do workspace, no gate): examples/ktu-embed com main.rs que mostra o caminho completo - ensure_layout (a porta Fs nao cria pastas), StdFs, SystemClock (adaptador Clock copiavel), StderrLogger (adaptador Logger), ToolThread+Toolbox, LlamaProvider ou um EchoProvider scripted (corre SEM modelo), Ktu::builder() e um turno com gravacao/retoma de sessao. Corrido: --backend scripted (EndTurn, sessao gravada), 2a corrida retoma (2 turn_start no mesmo log), --mode plan. O exemplo validou a ergonomia real (e o E0716 dos temporarios: &T::new() na cadeia nao compila - documentado com aviso nos snippets). Extendido o scripts/check_file_length.sh a examples/*/src. VERIFICACAO: os excertos canonicos (quickstart do README, receita 1, receita 4 com std::thread::scope) foram compilados num teste temporario e removidos depois; todos os links internos dos docs foram checados (resolvem)."
anchors: [wiki/integration/README.md, examples/ktu-embed/src/main.rs]
status: closed
scope: task
---
<!-- knudge:parent task_00dkc6dk -->