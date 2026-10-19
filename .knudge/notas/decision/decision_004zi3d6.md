---
id: decision_004zi3d6
type: decision
statement: Fila de mutacao vive na borda (FsTools) e a vez e pedida antes da leitura; o kernel nao conhece ficheiros
created_at: 2026-10-18T15:02:07.353Z
body_hash: c25d2a82
schema_version: 2
anchors: [crates/ktu-tools/src/mutation.rs, crates/ktu-tools/src/fs_tools.rs]
---
F4 (fila). Decisao: a FileMutationQueue e campo do FsTools (borda/T2, M2) e o edit_file pede a vez ANTES de ler. Motivo do sitio: o kernel e puro e nao conhece ficheiros; o Toolbox nao muda de assinatura e a fila fica com o mesmo tempo de vida da sessao (sem estado global). Motivo da ordem: a operacao de edicao e ler-modificar-gravar - se a leitura ficasse fora da fila, duas edicoes ao mesmo ficheiro liam o mesmo conteudo e a segunda gravacao apagava a primeira (lost update), mesmo com a fila a serializar as gravacoes. Alternativas descartadas: (a) lock global (um mutex para tudo) - serializa caminhos distintos sem necessidade, contra o criterio do gate; (b) fila no kernel - efeito no nucleo, M2; (c) lock por ficheiro em disco (flock) - mais uma porta e um efeito, e nao resolveria dentro do mesmo processo; (d) usar std::sync::Mutex dentro do async - bloquearia a T2 inteira em vez de esperar.