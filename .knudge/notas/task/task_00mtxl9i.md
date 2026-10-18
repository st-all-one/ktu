---
id: task_00mtxl9i
type: task
statement: A9 Receitas com settings/parameters/shell embutido (templates ricos) apos H12
created_at: 2026-10-18T02:58:07.131Z
body_hash: 31b6bbec
schema_version: 2
tags: [fase3]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T03:20:14.394Z
    notes: "A9 implementado (a pedido do dono, antes da H11): receitas ricas. CommandSettings{provider,base_url,model,max_steps} lidas do frontmatter (fail-closed em max_steps nao-inteiro); render com $1..$9 (posicionais, com aspas simples/duplas) + $ARGUMENTS e acrescento no fim quando nao ha marcador; ShellAccess{Disabled,Guarded} - shell: true e OPT-IN (sem ele !cmd e texto normal) e shell_segments() lista os segmentos; novo facade::evaluate_fact(spec, root, fs, fact) que avalia um facto isolado com o MESMO encadeamento da fachada (estrutural -> guardrails -> permissoes) - um so sitio constroi a cadeia (M4); ktu-cli/src/command_run.rs expande cada segmento so depois de Allow, usando a MESMA BashTool do turno (sem segundo motor), e falha com exit 1 + remedio accionavel em Deny/Escalate (verificado: bash never => \"shell negado (policy)\", ask => \"exige aprovacao\", always => corre); novo `ktu run --command <nome> [args]` com precedencia flag > comando > --params > config, e o REPL /nome usa o mesmo render (settings ignorados no REPL, com aviso); /help e os testes do slash passaram para ktu-cli/src/slash.rs (limite de ficheiro). Testes: command.rs 8 unit (incl. settings, posicionais, shell opt-in), ktu-cli/tests/commands.rs::run_command_applies_settings_and_guarded_shell. Continua fora: sub_recipes e parameters declarados (sem consumidor) - registado no plano H10."
anchors: [wiki/implementation_phase2/ADIADOS.md]
status: closed
scope: task
---
<!-- knudge:parent task_016wnwcs -->