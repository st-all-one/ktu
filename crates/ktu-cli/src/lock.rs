//! Lock de turno por sessão (`.ktu/sessions/<id>.lock`).
//!
//! Impede dois processos no mesmo id. A aquisição é **atómica** (`create_new`); um
//! lock **obsoleto** (mais de [`STALE_MS`]) é reaproveitado; um lock **vivo** faz o
//! segundo processo **recusar** — nunca apagamos um lock alheio.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use ktu_core::ports::Clock;

/// Idade a partir da qual um lock é considerado obsoleto.
pub(crate) const STALE_MS: i64 = 30_000;

/// Guarda do lock: remove o ficheiro ao ser largado.
#[derive(Debug)]
pub(crate) struct TurnLock {
    path: PathBuf,
}

impl TurnLock {
    /// Adquire o lock de `id` em `dir`; recusa se estiver **vivo**.
    ///
    /// # Errors
    /// Devolve mensagem legível se o lock estiver ocupado ou a escrita falhar.
    pub(crate) fn acquire(dir: &Path, id: &str, clock: &dyn Clock) -> Result<Self, String> {
        let path = dir.join(format!("{id}.lock"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        let now = clock.now().as_millis();
        match write_lock(&path, now) {
            Ok(()) => Ok(Self { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if is_stale(&path, now) {
                    let _ignored = fs::remove_file(&path);
                    write_lock(&path, now)
                        .map_err(|error| format!("{}: {error}", path.display()))?;
                    Ok(Self { path })
                } else {
                    Err(format!("sessao {id} ocupada por outro processo"))
                }
            }
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    }
}

impl Drop for TurnLock {
    fn drop(&mut self) {
        let _ignored = fs::remove_file(&self.path);
    }
}

/// Cria o lock em modo exclusivo (falha com `AlreadyExists` se já existir).
fn write_lock(path: &Path, now: i64) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    writeln!(file, "{now}")
}

/// `true` se o lock existente é mais antigo que [`STALE_MS`].
fn is_stale(path: &Path, now: i64) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return true;
    };
    let Ok(stamp) = text.trim().parse::<i64>() else {
        return true;
    };
    now.saturating_sub(stamp) > STALE_MS
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use std::process;

    use ktu_core::ports::{FixedClock, Timestamp};

    use super::{STALE_MS, TurnLock};

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("ktu-lock-{tag}-{}", process::id()));
        let _ignored = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn second_acquire_is_refused_until_released() {
        let dir = temp_dir("refuse");
        let clock = FixedClock::new(Timestamp::from_millis(1_000));
        let first = TurnLock::acquire(&dir, "s1", &clock);
        assert!(first.is_ok());
        assert!(TurnLock::acquire(&dir, "s1", &clock).is_err());
        drop(first);
        assert!(TurnLock::acquire(&dir, "s1", &clock).is_ok());
        let _ignored = fs::remove_dir_all(&dir);
    }

    #[test]
    fn stale_lock_is_reclaimed() -> io::Result<()> {
        let dir = temp_dir("stale");
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("s1.lock"), "1000\n")?;
        let later = FixedClock::new(Timestamp::from_millis(1_000 + STALE_MS + 1));
        assert!(TurnLock::acquire(&dir, "s1", &later).is_ok());
        let _ignored = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn distinct_ids_do_not_collide() {
        let dir = temp_dir("distinct");
        let clock = FixedClock::new(Timestamp::from_millis(1_000));
        let a = TurnLock::acquire(&dir, "a", &clock);
        let b = TurnLock::acquire(&dir, "b", &clock);
        assert!(a.is_ok() && b.is_ok());
        let _ignored = fs::remove_dir_all(&dir);
    }
}
