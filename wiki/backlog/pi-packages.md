# Pi packages (bundlar e distribuir recursos)

- **Estado:** candidata (do `pi`) — **avaliar**.
- **Origem:** `REF/pi/packages/coding-agent/docs/packages.md`; `wiki/revisao-pi.md` §2.5.
- **Âncora:** `.agents/`, `install.sh`.

## O que é
Um **pacote** (npm/git/local) que agrega **extensões + skills + templates + temas**, com
descoberta convencional (`extensions/`, `skills/`, `prompts/`, `themes/`) ou manifesto
explícito (`pi` key no `package.json`). Instalação `pi install`, listagem `pi list`.

## Porque interessa
O ktu já tem `.agents/{skills,guardrails}`; um conceito de **pacote** dá
distribuição/reutilização de recursos (skills, templates, guardrails). Usabilidade
objectiva (partilhar um "kit").

## O que o ktu tem hoje
`.agents/` no repositório; `examples/plugins/`; sem instalador de recursos.

## Pré-condições para reconsiderar
- Recursos a partilhar entre projectos.
- Transporte definido (**git/tarball**, não npm — o ktu é Rust).

## Custo e risco
- **Esforço:** médio (manifesto + instalador + trust).
- **Risco de não fazer:** baixo (copiar ficheiros serve).
- **Risco de fazer:** carregar recursos de terceiros sem confiança (ver `project-trust`).

## Decisão pendente
Adiar até haver recursos a partilhar. Se entrar, via **git/tarball** e com **trust**
explícito; nunca executar código de pacote sem confiança.
