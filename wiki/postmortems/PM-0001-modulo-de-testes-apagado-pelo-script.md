# PM-0001 — Módulo de testes apagado por um script de extracção

- **Data:** 2026-10-18 (H10/A9) · **Onde:** `crates/ktu-core/src/command.rs`

## Contexto

Para respeitar o limite de 400 linhas ([0011](../adr/0011-limite-de-400-linhas-e-firewall-mecanico.md)),
um módulo `#[cfg(test)] mod tests { … }` embutido foi extraído para `command/tests.rs` por um
pequeno script de uma vez só.

## Impacto

O script **reescreveu o ficheiro antigo primeiro** (`command.rs`, deixando lá `mod tests;`) e só
depois tentou criar `command/tests.rs` — mas o directório `command/` não existia e a escrita falhou.
Resultado: o crate não compilava (`mod tests;` sem ficheiro) e o **texto dos testes tinha
desaparecido** do ficheiro original. Os testes tiveram de ser reescritos à mão. O mesmo padrão
aconteceu **duas vezes** no mesmo dia (a segunda no mesmo ficheiro), porque a correcção da primeira
foi feita sem mudar o procedimento.

## Causa

Ordem de operações errada num script destrutivo: **modificar a origem antes de garantir o destino**
(não havia verificação de que o directório existia nem de que a escrita do destino sucedeu). E a
mensagem de erro (`FileNotFoundError`) apareceu *depois* de o dano estar feito.

## Correção

`crates/ktu-core/src/command/tests.rs` foi recriado (com os 8 testes: `parse`/`load`/`find`,
`settings`, argumentos posicionais e *shell* opt-in) e o crate voltou a compilar; o gate
(`make check`) ficou verde com 386 testes.

## Prevenção

- **Ordem**: criar o ficheiro de **destino** primeiro; só depois editar a origem (a origem é sempre
  recuperável do git enquanto o destino não existir).
- **Verificar depois de escrever**: correr o alvo mais pequeno possível (`cargo test -p <crate>
  --lib <modulo>`) imediatamente a seguir, antes de qualquer outro passo.
- O limite de linhas é agora satisfeito preferindo **módulos pequenos desde o início** (ex.:
  `command/tests.rs` e `slash.rs` nasceram como ficheiros), em vez de extracção retroactiva.
- **Restaurar também se verifica**: numa sessão de testes negativos (editar → correr → repor), uma
  cópia de segurança feita já sobre o ficheiro quebrado repõe a quebra. O que salva é correr o gate
  **no fim** (foi assim que o caso apareceu, minutos depois) — o estado final é o que conta, não a
  intenção do script.
