//! Media aritmetica.

/// Media de uma lista de numeros (lista vazia => 0).
pub fn media(valores: &[u32]) -> u32 {
    if valores.is_empty() {
        return 0;
    }
    let total: u32 = valores.iter().sum();
    total / u32::try_from(valores.len()).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::media;

    #[test]
    fn media_de_tres() {
        assert_eq!(media(&[2, 4, 6]), 4);
    }

    #[test]
    fn media_de_lista_vazia() {
        assert_eq!(media(&[]), 0);
    }
}
