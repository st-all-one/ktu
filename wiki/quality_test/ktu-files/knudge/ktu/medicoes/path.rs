use std::fs;
use std::time::Instant;

fn main() {
    let dir = "./bench-corpus/notas/fact";
    let start = Instant::now();
    let mut notes = 0usize;
    let mut keys = 0usize;
    let mut bytes = 0usize;
    let mut sum = 0u64;

    for entry in fs::read_dir(dir).expect("dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read");
        bytes += text.len();
        notes += 1;

        let mut lines = text.lines();
        if lines.next() == Some("---") {
            for line in lines {
                if line == "---" {
                    break;
                }
                if !line.starts_with(' ') && line.contains(':') {
                    keys += 1;
                }
            }
        }
        sum += text.as_bytes().iter().map(|&b| u64::from(b)).sum::<u64>();
    }

    let ms = start.elapsed().as_secs_f64() * 1000.0;
    println!(
        "rust  notas={notes} chaves={keys} bytes={bytes} soma={sum} total={ms:.2}ms por_nota={:.1}us",
        ms * 1000.0 / notes as f64
    );
}
