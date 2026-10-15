---
question_id: Q1
question_issue: "faça um resumo do repositório /REF/knudge de modo que uma criança de 5 anos possa entender"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: opencode 
---

# 🧠 O que é o knudge? (explicado para uma criança de 5 anos)

Imagine que você tem um **caderninho mágico** para guardar as coisas que você aprende.

Quando você brinca de montar um castelo de blocos, às vezes esquece como encaixar as peças.
Aí você pergunta para o seu caderninho, e ele te responde: *"Você já descobriu que a peça azul
vai antes da vermelha!"* Pronto, você não precisa descobrir tudo de novo.

É isso que o **knudge** faz — mas o "você" aqui é um **robô ajudante de computador** (a gente
chama de *agente de IA*). O knudge é o **caderninho mágico dele**.

---

## 📒 As ideias principais (bem simples)

### 1. O caderninho de papéis é o que importa
O robô escreve cada coisa que aprende numa **folhinha de papel** (um arquivo de texto simples,
chamado *Markdown*). Essas folhinhas são a **verdade** — são elas que guardam o conhecimento de
verdade. Ninguém joga fora.

> 🧒 Regra de ouro: **"a folhinha de papel manda; todo o resto é ajuda para achar as folhinhas".**

### 2. A lista de atalhos pode ser refeita
Para achar as folhinhas rápido, o knudge monta uma **listinha de atalhos** (um *índice*, tipo o
índice de um livro). Se essa listinha se perder... **nada de ruim acontece!** Basta olhar as
folhinhas de novo e refazer os atalhos. É como apagar o índice de um livro e escrever outro:
as histórias continuam lá. 😊

### 3. O caderninho acha as coisas de vários jeitos
Quando o robô pergunta *"como é que eu faço isso mesmo?"*, o knudge procura:
- pelas **palavrinhas** (como quando você procura "dinossauro" no livro),
- pelas **marcas que apontam para o código** (tipo setinhas dizendo "isto combina com aquela parte"),
- e, se quiser, por **um truque esperto** que entende o *significado* das coisas, não só as letras.

Depois de procurar de todos os jeitos, ele junta tudo e diz a **melhor resposta primeiro**. 🏆

### 4. Ele aprende e não repete
Antes de escrever algo novo no caderninho, o robô **pergunta primeiro**: *"Será que já sei isso?"*
- Se **nunca aprendeu**, escreve uma folhinha nova. ✍️
- Se é **quase igual** ao que já sabia, ele **junta** na mesma folhinha. 🤝
- Se é **exatamente a mesma coisa**, ele diz *"já sei isso!"* e **não escreve de novo**. 🙅

### 5. O robô tem uma lista de tarefas
Além das folhinhas de conhecimento, o knudge guarda uma **listinha de afazeres**:
grandes objetivos (tipo "construir o castelo 🏰"), problemas no meio do caminho e as tarefinhas
pequeninas. Uma tarefa só fica pronta quando o robô mostra **uma prova** de que deu certo
(não vale dizer "terminei!" sem mostrar).

### 6. Ele guarda o lugar onde parou
Quando o robô vai dormir (termina de trabalhar), ele deixa um **marcador de página** dizendo onde
parou. Assim, quando acordar, sabe continuar do jeitinho certo.

---

## 🎮 Como o robô usa (o ciclo mágico)

O robô sempre faz a mesma dancinha: **perguntar → escrever → fazer tarefa → guardar**.

1. 🔍 `kd ask` — *"Caderninho, o que você sabe sobre isso?"* (pergunta **antes** de fazer).
2. ✍️ `kd write` — *"Caderninho, escreve que eu aprendi isto!"* (uma folhinha por aprendizado).
3. ✅ `kd task` — *"O que eu preciso fazer agora?"* (a listinha de afazeres).
4. 🔖 `kd rewind` / `kd sync` — *"Marca onde parei e guarda tudo."* (marcador + versionamento).

O comando é o **`kd`** (é o nome do brinquedo na linha de comando).

---

## 🧩 Como é feito por dentro (a casa do knudge)

O knudge é feito de **três caixinhas** que trabalham juntas (escritas numa linguagem de computador
chamada **Rust**, bem rápida e segura):

- 📦 **`knudge-core`** — a **cabeça** 🧠. Pensa, guarda o formato das folhinhas, faz a busca. É a
  parte mais "pura": não mexe no computador de verdade sozinha.
- 📦 **`knudge-cli`** — o **robô `kd`** 🤖. É com quem você conversa no teclado.
- 📦 **`knudge-mcp`** — a **porta de trás** 🔌. Deixa outros robôs espertos (a IA) olharem o
  caderninho sozinhos, sem você precisar pedir.

A "cabeça" só pede ajuda para as **caixinhas externas** (o relógio ⏰, os arquivos 📁, o git
🌳...) quando realmente precisa. Assim ela é fácil de testar e nunca faz bagunça escondida.

---

## 🏆 Por que isso é legal?

- **Nunca esquece** o combinado entre uma sessão e outra. 🐘
- **Você pode espiar e auditar** tudo depois — são só papéis escritos. 👀
- **Não precisa de servidor, banco de dados ou robô de plantão** — é um brinquedinho simples e
  leve. 🪶
- **Não perde nada**: se os atalhos quebrarem, é só refazer. 💪
- É **bem testado** e organizado (centenas de testes, arquivinhos pequenininhos de no máximo
  300 linhas) para nunca fazer travessuras. ✨

---

## 📊 Ficha rápida

| Coisa | Valor |
|---|---|
| Nome | **knudge** (comando `kd`) |
| O que é | Caderninho mágico de memória para robôs de IA, por projeto |
| Feito em | Rust (precisa do Rust 1.97 ou mais novo) |
| Versão | 0.5.2 |
| Cabeça | `knudge-core` · Robô do teclado: `knudge-cli` (`kd`) · Porta traseira: `knudge-mcp` |
| Tamanho | Uns 57 mil versinhos de código Rust |
| Licença | MIT ou Apache-2.0 (uso livre 🎈) |
| Lema | *"A memória que o seu agente consulta antes de decidir — e que você consegue auditar depois."* |

### Em uma frase 🍼
> O knudge é o **caderninho de papel do robô**: ele anota, lembra, procura, não repete as coisas e
> deixa até você espiar tudo depois — e se a listinha de atalhos cair no chão, é só refazer. 😄
