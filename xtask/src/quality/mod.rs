//! `xtask quality` — bateria de qualidade `p1`–`p4` com o ktu como sujeito
//! isolado (F5/G4).
//!
//! Lê as perguntas, planifica a sequência, corre cada turno contra o `ktu`,
//! arquiva os artefactos ao lado dos referenciais e sonda a persistência entre
//! sessões.

mod artifact;
mod exec;
mod frontmatter;
mod options;
mod plan;
mod workspace;

use std::io::{self, Write};

use exec::{run_battery, write_plan};
use options::Options;
use plan::{Question, Step};

/// Corre a bateria de qualidade.
///
/// # Errors
/// Propaga erros de opção, leitura, execução do `ktu` ou escrita dos artefactos.
pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let options = Options::parse(args)?;
    let questions = load_questions(&options)?;
    let steps = plan::battery(&questions, &options.subject);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    if options.dry_run {
        return write_plan(&mut out, &options, &steps);
    }
    execute(&mut out, &options, &questions, &steps)
}

/// Carrega as perguntas selecionadas (erro se alguma faltar).
fn load_questions(options: &Options) -> Result<Vec<Question>, String> {
    let mut questions = Vec::new();
    for key in options.keys() {
        questions.push(Question::load(&options.root, &options.questions, &key)?);
    }
    if questions.is_empty() {
        return Err("nenhuma questao selecionada".to_owned());
    }
    Ok(questions)
}

/// Executa a bateria com isolamento: prepara o espaço, corre e arquiva.
fn execute(
    out: &mut dyn Write,
    options: &Options,
    questions: &[Question],
    steps: &[Step],
) -> Result<(), String> {
    workspace::prepare(&options.root, &options.work)?;
    run_battery(out, options, questions, steps)?;
    workspace::archive(&options.work, &options.output, &options.subject)
}
