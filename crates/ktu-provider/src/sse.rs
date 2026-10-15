//! Framing de *Server-Sent Events* (SSE) — puro e incremental.
//!
//! O `llama-server` envia linhas `data: {...}` separadas por uma linha vazia
//! (`\n\n`) e termina com `data: [DONE]`. O framer acumula bytes parciais e só
//! devolve *payloads* completos.

/// Acumulador de bytes SSE.
#[derive(Debug, Default)]
pub struct SseFramer {
    buffer: String,
}

impl SseFramer {
    /// Cria um framer vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Acrescenta bytes recebidos e devolve os *payloads* `data:` completos.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.push_str(&String::from_utf8_lossy(chunk));
        if self.buffer.contains('\r') {
            self.buffer = self.buffer.replace("\r\n", "\n");
        }
        let mut payloads = Vec::new();
        while let Some(index) = self.buffer.find("\n\n") {
            let frame = self.buffer.get(..index).unwrap_or("").to_owned();
            let consumed = index.saturating_add(2);
            self.buffer = self.buffer.get(consumed..).unwrap_or("").to_owned();
            for line in frame.lines() {
                if let Some(data) = line.strip_prefix("data:") {
                    payloads.push(data.trim().to_owned());
                }
            }
        }
        payloads
    }
}

#[cfg(test)]
mod tests {
    use super::SseFramer;

    #[test]
    fn frames_split_across_chunks() {
        let mut framer = SseFramer::new();
        assert_eq!(framer.push(b"data: {\"a\":1}"), Vec::<String>::new());
        let payloads = framer.push(b"\n\ndata: [DONE]\n\n");
        assert_eq!(payloads, vec!["{\"a\":1}".to_owned(), "[DONE]".to_owned()]);
    }

    #[test]
    fn handles_crlf_and_multiple_events_in_one_chunk() {
        let mut framer = SseFramer::new();
        let payloads = framer.push(b"data: um\r\n\r\ndata: dois\r\n\r\n");
        assert_eq!(payloads, vec!["um".to_owned(), "dois".to_owned()]);
    }
}
