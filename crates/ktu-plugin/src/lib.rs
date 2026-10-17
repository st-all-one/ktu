//! `ktu-plugin` — runtime de plugins externos (§2.9).
//!
//! Um plugin é um **processo externo** que fala JSON-lines sobre stdio: recebe
//! `{id, event, payload}` e responde `{verdict}`. O kernel só conhece a porta
//! [`ktu_core::plugin::Hooks`]; este crate é o adaptador de efeitos (borda).
#![forbid(unsafe_code)]

pub mod host;
pub mod manifest;
pub mod process;

pub use host::PluginHost;
pub use manifest::Manifest;
pub use process::ProcessPlugin;
