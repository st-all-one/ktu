---
question_id: Q1
question_issue: "faça um resumo do repositório /REF/knudge de modo que uma criança de 5 anos possa entender"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: pi 
---

# knudge, explicadinho para uma criança de 5 anos 🧸

## O que é? 🎒

Imagina que você tem um **caderninho mágico** de todas as coisas do seu projeto.

Quando você tem uma ideia, você **anota no caderninho**. Depois, dias depois,
quando esquecer, você **pergunta para o caderninho** e ele te conta de novo!
Ele nunca esquece nada. 😊

O nome desse caderninho é **knudge** ("nudge" = um cutucão, tipo "ei, lembra disso!").
Ele ajuda os **robôs que escrevem código** (os "agentes de IA") a lembrar das coisas,
igualzinho você lembra que o elefante é grande. 🐘

## Como ele guarda as coisas? 📒

- Cada coisinha que a gente sabe vira uma **folhinha de papel** (uma "nota").
- As folhinhas ficam todas juntas, numa **pastinha chamada `notas/`**.
- As folhinhas são escritas do jeito que a gente entende: **em Markdown**, que é
  texto simples com titulozinhos e estrelinhas. ✏️
- E tem um **índice** — igual aquela listinha no começo de um livro que diz
  "leão... página 7". Esse índice ajuda a **achar as folhinhas bem rápido**.
- O legal: se o índice perder, sem problema! A gente monta ele de novo olhando
  só as folhinhas. As folhinhas são as **de verdade**. 🥇

## O que tem dentro da caixa? 📦

O knudge é como uma caixa com **três brinquedos**:

1. **`kd`** — o **brinquedo principal**. É com ele que a gente conversa:
   pergunta, anota, faz listinha de tarefas. 🕹️
2. **`knudge-core`** — o **motorzinho escondido** lá dentro. Faz o trabalho pesado,
   quietinho. ⚙️
3. **`knudge-mcp`** — um **telefone** que deixa o robô falar com o caderninho
   sozinho, sem você pedir. ☎️

> É feito com uma linguagem de computador chamada **Rust** — que é forte e rápida
> como um caminhão de bombeiro. 🚒

## O que a gente faz com ele? (a musiquinha) 🎵

Tem uma ordem de passos, tipo uma musiquinha:

1. **Perguntar** 🔍 — "Já sabemos disso?" (procura no caderninho **antes** de escrever)
2. **Anotar** ✏️ — "Ah, descobri! Vou guardar."
3. **Fazer tarefas** ✅ — "O que falta fazer?"
4. **Guardar tudo** 💾 — "Salva no cofre pra não perder."

É igual quando você:
- **pergunta** pra mamãe se pode,
- **escreve** um desenho,
- **faz** o desenho,
- **guarda** na geladeira para mostrar pro papai. 🧑‍🎨

## Regrinhas do caderninho 📏

- 🔎 **Olhe primeiro, escreva depois!** Nunca anote algo que já está lá. Senão o
  caderninho fica bagunçado, cheio de coisas repetidas.
- 📝 **Uma folhinha, uma ideia só.** Não coloca tudo junto. É como não misturar
  o almoço com a sobremesa. 🍝🍨
- 🧷 **Prende a folhinha no lugar certo.** Se é sobre um arquivo do projeto,
  a folhinha fica **grudadinha** nele. Aí fica fácil achar.
- 🎈 **Nada some.** Pegou uma folhinha antiga? Ela vira uma folhinha **nova melhor**,
  mas a antiga fica guardada. Nada de apagar com borracha! 🧽

## Como um robô usa o caderninho? 🤖

Os robôs são meio **esquecidinhos** — toda hora eles "nascem de novo" e esquecem
tudo que aprenderam. 😵

O knudge é o **caderninho que o robô abre antes de decidir as coisas**. Assim ele
não esquece que "o portão só deixa passar 100 pessoas por segundo", por exemplo.
E você pode **espiar no caderninho depois** para ver o que o robô anotou — nada de
segredo! 🔓

## Guarda-se onde? 🏠

No **seu próprio computador**, dentro de uma pastinha do projeto (chamada `.knudge/`).
Não precisa de internet, nem de mágica grande, nem de robô gigante na nuvem.
É tudo ali, do ladinho. 🏡

## Resumindo bem curtinho 🎁

> **knudge = o caderninho de lembranças do projeto.**
> Pergunta antes de anotar, não deixa nada sumir, e ajuda o robô a não esquecer.
> A folhinha é a verdade; o índice é só o atalho. 📒➡️🔍
