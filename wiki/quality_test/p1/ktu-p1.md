---
question_id: Q1
question_issue: "faça um resumo do repositório /REF/knudge de modo que uma criança de 5 anos possa entender"
expected_answer_ideia: ""
provider: opencode-go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
same_session: false
compacted: false
agent: ktu
---
# 🧸 O knudge, contado como uma história

Imagina que tens um **robô amigo** que te ajuda a fazer coisas no computador. O robô é esperto, mas tem um problema: **esquece-se de tudo** quando vai dormir. 😴

O **knudge** (que se lê mais ou menos "nudge") é uma **caixinha mágica de memória** para esse robô. Ele fica no projeto e ajuda o robô a **lembrar-se** das coisas entre um dia e o outro.

---

## 📒 Como é que a memória funciona?

### 1. Tem cartõezinhos 📝
Cada coisa que se aprende vai para um **cartãozinho**. Cada cartãozinho diz **só uma coisa**. Assim é fácil de encontrar.

Por exemplo:
- *"A porta do castelo abre-se com a chave azul"* 🔑
- *"O dragão mora na gruta grande"* 🐉

Há cartões de vários tipos: uns dizem **factos**, outros dizem **decisões** (escolhas feitas), outros avisam de **perigos**, outros guardam **perguntas** que ainda não têm resposta.

### 2. Os cartões verdadeiros são guardados numa caixa boa 🗃️
Os cartões são ficheiros simples (como folhas de papel com uma etiqueta no topo). Eles são a **verdade**. Se estragar tudo o resto, os cartões continuam lá. ✔️

### 3. Existe também um mapa do tesouro 🗺️
Para encontrar um cartão rapidamente, o knudge faz um **mapa** — como um índice de um livro grande. Mas atenção: o mapa é só um **ajudante**. Se o mapa se rasgar, **não perdemos nada**: faz-se um mapa novo a partir dos cartões. 🎨

### 4. Fios que ligam os cartões ao código 🧵
Cada cartão pode ter uns **fios** (chamamos-lhes *âncoras*) que apontam para o sítio do programa donde veio aquela ideia. Se um fio apontar para uma parede que já não existe, o knudge avisa: *"ó, este fio está partido!"* ✂️

### 5. O robô pergunta antes de guardar 🙋
Antes de escrever um cartão, o robô **procura** se já existe um igual:
- Não existe nada parecido? **Escreve um cartão novo.** 🆕
- Já existe um parecido? **Junta tudo num só** (para não haver dois cartões iguais a dizer o mesmo). 🤝
- Já existe exatamente o mesmo? **Não escreve nada.** ✋

Isto é importante, senão a caixa enchia-se de mil papéis a repetir a mesma coisa!

---

## 🔄 O que o robô faz todos os dias: 4 passos

| Passo | O que é | Como se faz |
|---|---|---|
| 👀 **Procurar** | *"O que é que já sei sobre isto?"* | `kd ask` |
| ✍️ **Escrever** | *"O que é que eu aprendi hoje?"* | `kd write` |
| 🧹 **Trabalhar** | *"O que é que falta fazer?"* | `kd task` |
| 📤 **Guardar no baú partilhado** | *"Vou guardar para os amigos também verem"* | `kd sync` |

Palavra de ordem da caixinha: **procura antes de escrever!** 🔍

---

## ✅ A lista de tarefas (com caixas dentro de caixas)

O robô também tem uma lista do que falta fazer, organizada como **bonecas russas** 🪆:

- 📦 **Caixa grande** (épico) — o trabalho todo, tipo *"fazer um bolo"*.
- 🎁 **Caixa média** (issue) — uma parte, tipo *"fazer o recheio"* (e às vezes nem é preciso).
- 🍬 **Caixinha pequena** (tarefa) — uma coisinha, tipo *"partir os ovos"*.

E qual é a regra de ouro? **Só se pode fechar uma tarefa se mostrarmos a prova!** Não basta dizer *"está feito"* — tens de mostrar o bolo. 🍰 (`kd task close`)

---

## 🔒 Regras boas que o knudge promete

- 🚫 **Nunca apaga nada à força.** Se um cartão já não serve, põe-lhe um autocolante a dizer *"esquecido"* — mas o cartão fica lá, para podermos ver depois o que aconteceu.
- 🙏 **Nada muda sem pedir licença.** Quando o knudge quer limpar coisas velhas, só **sugere**: *"olha, achava que este cartão já não presta"*. Quem decide és tu.
- 🤐 **Nunca estraga o teu desenho.** Quando te dá uma resposta, escreve os **dados** numa folha e as **conversas** noutra. Nunca mistura tudo num borrão.
- 🧭 **Número é número.** Se fizermos as mesmas perguntas duas vezes, a resposta é **sempre a mesma** — nada de respostas diferentes conforme o vento. 🌬️

---

## 👥 Quem são os amigos do knudge?

- 🐶 **`kd`** — o cão-trajeto da caixinha. É com ele que falas: `kd ask`, `kd write`, `kd task`… (se não lhe disseres nada, ele mostra o menu de ajuda).
- 🤖 **`knudge-mcp`** — o **telefone especial** que liga a caixinha ao robô de IA. Quando vais escrever algo, o robô ouve um *"piu piu"*: *"espera, olha este cartão primeiro!"* 📞 Mas ele só recebe **bilhetinhos curtos** (o número do cartão e a primeira linha) — nunca o cartão todo, para não ficar com a cabeça cheia. 🤯

---

## 🏠 Onde é que isto vive? (a parte de "crescidos")

Está numa linguagem chamada **Rust**, como um brinquedo bem construído com peças apertadas — difícil de partir. 🧱 Tem três partes:

1. **`knudge-core`** — o cérebro 🧠 (as regras).
2. **`knudge-cli`** — a boca 👄 (é aí que nasce o comando `kd`).
3. **`knudge-mcp`** — o ouvido 📞 (fala com os robôs de IA).

E os construtores têm regras para o brinquedo nunca rebentar: **peças pequenas** (cada ficheiro no máximo 300 linhas) e **proibido gritar/rebentar** (nada de `panic`). 💥❌

---

### 🗝️ Dicionário de crescidos → palavras de criança

| Palavra difícil | Palavras de criança |
|---|---|
| *memória durável* | caderno que não se apaga |
| *nota* | cartãozinho |
| *índice derivado* | mapa do tesouro que se pode refazer |
| *busca híbrida* | procurar no mapa E nos cartões, ao mesmo tempo |
| *dedup* | juntar papéis iguais para não haver repetidos |
| *âncora* | fio que liga o cartão ao desenho do programa |
| *sync* | guardar no baú para os amigos verem |

---

**Resumindo numa frase:** o knudge é a **caixinha mágica onde o robô guarda cartõezinhos com aquilo que aprendeu** — procura antes de escrever, junta os iguais, liga cada ideia ao sítio certo, e nunca apaga nada sem pedir autorização. 🎈

*Baseei-me no `README.md`, no `llms.txt` e nas specs/uso em `REF/knudge/wiki` (docs do próprio repositório).*
