---
question_id: Q1
question_issue: "faça um resumo do repositório /REF/knudge de modo que uma criança de 5 anos possa entender"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: goose 
---

# 🧠 O knudge explicado para uma criança de 5 anos

> Uma historinha para entender o que é esse repositório. Sem palavras difíceis. 💛

---

## 🐣 Imagina assim...

Você tem um **amigo robô** 🤖 que ajuda você a construir coisas (brincar de LEGO, montar
quebra-cabeças, fazer um desenho bem grande).

O problema é que esse robô **esquece tudo** quando você sai da sala! Aí no dia seguinte ele
pergunta de novo:

> "O que a gente estava fazendo mesmo?"

Isso deixa todo mundo cansado, porque ele fica **descobrindo a mesma coisa** mil vezes. 😵

O **knudge** é o **caderninho de anotações** desse robô! 📓

---

## 📓 Como funciona o caderninho

### 1. O robô anota tudo o que aprende ✏️

Cada vez que o robô aprende uma coisinha nova, ele escreve **uma notinha** no caderninho.

> Exemplo: *"A porta da cozinha abre com 100 voltinhas."*

Uma nota = uma ideia só. Curtinha e fácil de achar. Nunca escreve duas ideias na mesma nota,
porque fica bagunçado!

### 2. Antes de responder, ele lê o caderninho 👀

Se você pergunta alguma coisa, o robô **primeiro procura** no caderninho se ele já sabe a
resposta. Assim ele **não chuta** e não inventa bobagem.

### 3. Antes de anotar, ele procura se já tem igual 🔍

O robô é esperto: antes de escrever uma nota nova, ele olha se já existe uma parecida.

- Se **não tem nada parecido** → ele escreve a nota nova. ✅
- Se **tem uma quase igual** → ele junta as duas. 🤝
- Se **já tem uma igualzinha** → ele não escreve nada. 🚫

Assim o caderninho **nunca fica cheio de bobagem repetida**!

---

## 🎒 As quatro coisinhas que o robô sabe fazer

| O que ele faz | Como se chama | É tipo... |
|---|---|---|
| **Perguntar** o que já sabe | `ask` | "Ô caderninho, você sabe isso?" 🙋 |
| **Anotar** o que aprendeu | `write` | Escrever com lápis ✏️ |
| **Planejar** o que falta fazer | `task` | Uma listinha de tarefas 📝 |
| **Guardar** tudo direitinho | `sync` | Colocar o caderno na mochila 🎒 |

E ele faz isso sempre na mesma ordem:

**pergunta ➜ anota ➜ planeja ➜ guarda**
`ask` ➜ `write` ➜ `task` ➜ `sync`

---

## 📚 O truque do "livro com índice"

Sabe quando você tem um livro grande e ele tem aquele **índice** no começo, para você achar
a página rápido? 📖

O knudge é igual:

- As **notas** (os textos) são o **livro de verdade**. Esse é o tesouro! 💎
- O **índice** é só um **atalho** para achar rápido.

E o melhor: **se o índice se perder, não tem problema!** O robô gera o índice de novo, sozinho,
olhando as notas. O tesouro nunca se perde. 😌

> Por isso o pessoal do projeto gosta de dizer:
> **"A nota é a verdade, o índice é só um atalho."**

---

## 🏷️ As etiquetas que gruda no código

O robô tem umas **etiquetas** (tipo aquelas de nome, `--anchor`) para ligar cada notinha a uma
partezinha do projeto.

Pensa num varal de roupas: cada pregador 🗜️ segura a roupa no lugar certo. Se a roupa cai
(o código muda), o robô percebe e arruma o pregador. 🧺

---

## 🔌 O "plugue mágico"

Tem um plugue especial chamado **`knudge-mcp`**. É tipo o **cabo do carregador** 🔌: ele liga
o robô ao caderninho para que tudo funcione junto, sem ninguém precisar apertar botões.

O comando mágico que você digita se chama **`kd`** — é o "abre-te-sésamo" do caderninho. ✨

---

## 🗂️ O que tem dentro da caixinha do projeto

Imagina que o projeto é uma **caixa de brinquedos** organizada em gavetinhas:

| Gavetinha | O que tem dentro |
|---|---|
| `crates/knudge-core` | O **cérebro** do robô (pensa e guarda as coisas) 🧠 |
| `crates/knudge-cli` | A **boca** que fala com você (o comando `kd`) 🗣️ |
| `crates/knudge-mcp` | O **plugue** que liga no robô 🔌 |
| `wiki/` | Os **livros de instruções** para gente grande 📚 |
| `plan/` | O **caderno de planos** de como ele foi construído 🗺️ |
| `bench/` | Os **testes e corridas** para ver se está rápido 🏃 |

---

## 🌟 Numa frase só

> **O knudge é um caderninho mágico que ajuda o robô ajudante a não esquecer nada do seu
> projeto — ele anota o que aprende, procura antes de responder e guarda tudo em textinhos
> fáceis de ler.** 📓🤖💛

E quando você fecha o computador, o caderninho fica guardado junto com o projeto, para o robô
lembrar de tudo no dia seguinte! 🎉

---

*Feito com carinho pelo goose 🪿*
