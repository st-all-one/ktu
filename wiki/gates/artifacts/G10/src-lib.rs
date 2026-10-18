//! Alvo do gate G10: modulo pequeno, com um bug e um teste que falha.

pub mod media;

/// Soma dois numeros.
pub fn soma(a: u32, b: u32) -> u32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::soma;

    #[test]
    fn soma_dois_numeros() {
        assert_eq!(soma(2, 3), 5);
    }
}
