//! Valores por omissão por provider (base URL e modelo).

/// *Base URL* por omissão do provider.
#[must_use]
pub(crate) fn default_base(provider: &str) -> &'static str {
    match provider {
        "opencode-go" => "https://opencode.ai/zen/go",
        _ => "http://127.0.0.1:8080",
    }
}

/// Modelo por omissão do provider.
#[must_use]
pub(crate) fn default_model(provider: &str) -> &'static str {
    match provider {
        "opencode-go" => "deepseek-v4.1-flash",
        _ => "ornith",
    }
}

#[cfg(test)]
mod tests {
    use super::{default_base, default_model};

    #[test]
    fn opencode_go_defaults_point_to_the_zen_go_gateway() {
        // A base não inclui `/v1`: o endpoint acrescenta `/v1/chat/completions`.
        assert_eq!(default_base("opencode-go"), "https://opencode.ai/zen/go");
        assert_eq!(default_model("opencode-go"), "deepseek-v4.1-flash");
    }

    #[test]
    fn unknown_provider_falls_back_to_local_llama() {
        assert_eq!(default_base("llama"), "http://127.0.0.1:8080");
        assert_eq!(default_model("llama"), "ornith");
    }
}
