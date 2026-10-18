# 0001 — Kernel puro com portas; efeitos só na borda

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/ports/`, `xtask/src/main.rs` (`check-layers`)

## Contexto

A 1.ª tentativa (`REF/katu`) tinha 70k linhas e nenhuma sessão agêntica estável: o loop estava
entrelaçado com terminal, relógio, FS e rede, o que tornava impossível testá-lo sem um ambiente
completo. O ktu começa pela pergunta inversa: *o que é o loop, se lhe tirarmos todos os efeitos?*

## Decisão

O `ktu-core` é **domínio puro**: não toca no FS, no relógio, no RNG global, no terminal, em `argv`
nem na rede. Tudo o que precisa vem por **porta** (`Fs`, `Clock`, `Logger`, `Metrics`, `Embedder`,
`Instructions`), injectada pelo chamador. O `xtask check-layers` falha se um crate interno importar
uma dependência proibida (ex.: `ktu-core` → `ktu-tools`).

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Kernel com acesso directo ao FS/relógio (mais simples de escrever) | foi o erro do katu: o loop deixa de ser testável sem ambiente; `MemFs`/`FixedClock` desaparecem |
| Traits `async` em todas as portas desde o início | obrigaria `async_trait` em todas as fronteiras e a cor do `async` a subir para o domínio sem necessidade |
| Um crate único com módulos separados por convenção | a firewall deixa de ser mecânica e passa a ser disciplina (ver [0011](0011-limite-de-400-linhas-e-firewall-mecanico.md)) |
| `Env`/`Rng` como portas desde já | declaradas sem consumidor no loop — ficaram como portas opcionais (YAGNI, [0015](0015-yagni-com-criterio-de-reabertura.md)) |

## Consequências

- O loop corre inteiro em memória, com `MemFs` + `FixedClock` + provider/tools falsos (é assim que
  os testes de `crates/ktu-core/tests/loop.rs` provam I1–I17).
- Os adaptadores reais vivem nos crates de borda (`ktu-tools`, `ktu-provider`, `ktu-cli`).
- Custo: mais tipos de fronteira e `Deps`/`Capabilities` explícitos; em troca, o kernel é
  reconstruível e o determinismo (I6) é testável.
