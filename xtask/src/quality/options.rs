//! Opções de linha de comandos de `xtask quality`.

use std::path::PathBuf;
use std::slice::Iter;

/// Chaves padrão da bateria.
pub(super) const DEFAULT_KEYS: [&str; 4] = ["p1", "p2", "p3", "p4"];

/// Opções de linha de comandos de `xtask quality`.
#[derive(Clone, Debug)]
pub(super) struct Options {
    /// Raiz do projeto onde o `ktu` corre.
    pub(super) root: PathBuf,
    /// Caminho do binário `ktu`.
    pub(super) ktu: PathBuf,
    /// Nome do sujeito (`agent`).
    pub(super) subject: String,
    /// Provider do sujeito.
    pub(super) provider: String,
    /// Modelo do sujeito.
    pub(super) model: String,
    /// Esforço de raciocínio (`thinking-level`).
    pub(super) effort: String,
    /// *Base URL* do `llama-server`.
    pub(super) base_url: String,
    /// Tecto de passos por turno.
    pub(super) max_steps: String,
    /// Diretório dos artefactos (`wiki/quality_test`).
    pub(super) output: PathBuf,
    /// Diretório de onde se leem as perguntas (`wiki/quality_test`).
    pub(super) questions: PathBuf,
    /// Subconjunto de questões a correr (`p1,p2`).
    pub(super) only: Vec<String>,
    /// Só imprime o plano, sem correr o modelo.
    pub(super) dry_run: bool,
}

impl Options {
    /// Valores por omissão da bateria.
    fn new() -> Self {
        Self {
            root: PathBuf::from("."),
            ktu: PathBuf::from("target/debug/ktu"),
            subject: "ktu".to_owned(),
            provider: "llama".to_owned(),
            model: "ornith".to_owned(),
            effort: "xhigh".to_owned(),
            base_url: "http://127.0.0.1:8080".to_owned(),
            max_steps: "24".to_owned(),
            output: PathBuf::from("wiki/quality_test"),
            questions: PathBuf::from("wiki/quality_test"),
            only: Vec::new(),
            dry_run: false,
        }
    }

    /// Interpreta os argumentos de `xtask quality`.
    ///
    /// # Errors
    /// Devolve mensagem para opção desconhecida ou valor em falta.
    pub(super) fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self::new();
        let mut iter = args.iter();
        while let Some(flag) = iter.next() {
            match flag.as_str() {
                "--root" => options.root = PathBuf::from(value(&mut iter, flag)?),
                "--ktu" => options.ktu = PathBuf::from(value(&mut iter, flag)?),
                "--subject" => options.subject = value(&mut iter, flag)?,
                "--provider" => options.provider = value(&mut iter, flag)?,
                "--model" => options.model = value(&mut iter, flag)?,
                "--effort" => options.effort = value(&mut iter, flag)?,
                "--base-url" => options.base_url = value(&mut iter, flag)?,
                "--max-steps" => options.max_steps = value(&mut iter, flag)?,
                "--output" => options.output = PathBuf::from(value(&mut iter, flag)?),
                "--questions" => options.questions = PathBuf::from(value(&mut iter, flag)?),
                "--only" => options.only = split_keys(&value(&mut iter, flag)?),
                "--dry-run" => options.dry_run = true,
                other => return Err(format!("opcao desconhecida: {other}")),
            }
        }
        Ok(options)
    }

    /// Chaves da bateria efetivamente selecionadas.
    pub(super) fn keys(&self) -> Vec<String> {
        if self.only.is_empty() {
            DEFAULT_KEYS.iter().map(|key| (*key).to_owned()).collect()
        } else {
            self.only.clone()
        }
    }
}

/// Lê o valor seguinte a uma flag.
fn value(iter: &mut Iter<'_, String>, flag: &str) -> Result<String, String> {
    iter.next()
        .cloned()
        .ok_or_else(|| format!("{flag} exige um valor"))
}

/// Divide uma lista por vírgulas, ignorando espaços e vazios.
fn split_keys(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|part| part.trim().to_owned())
        .filter(|part| !part.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Options, split_keys};

    #[test]
    fn split_keys_ignores_blanks() {
        assert_eq!(split_keys("p1, p2 ,,p3"), vec!["p1", "p2", "p3"]);
    }

    #[test]
    fn parse_sets_subject_and_only() {
        let args = vec![
            "--subject".to_owned(),
            "ktu".to_owned(),
            "--only".to_owned(),
            "p1,p2".to_owned(),
        ];
        let Ok(options) = Options::parse(&args) else {
            return;
        };
        assert_eq!(options.subject, "ktu");
        assert_eq!(options.keys(), vec!["p1", "p2"]);
    }
}
