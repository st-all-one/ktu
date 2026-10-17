// Gera um corpus sintético no formato do knudge para medir o data path.
import { mkdirSync, writeFileSync } from 'node:fs';

const N = Number(process.argv[2] ?? 1000);
mkdirSync('bench-corpus/notas/fact', { recursive: true });

let body = '';
for (let i = 0; i < 12; i++) {
  body += `A decisao ${i} descreve o comportamento do sistema sob concorrencia e persistencia, ` +
    `com justificativa, impacto e bordas conhecidas. Termo${i} aparece aqui para densidade lexical.\n\n`;
}

for (let i = 0; i < N; i++) {
  const id = `fact_${(i * 2654435761 % 2176782336).toString(36).padStart(6, '0')}${i % 100}`;
  const front = [
    `id: ${id}`,
    'type: fact',
    `statement: Decisao ${i} sobre persistencia e concorrencia do sistema`,
    'created_at: 2026-01-02T03:04:05.678Z',
    'confidence: 0.7',
    `body_hash: ${(i * 1000003).toString(16).padStart(8, '0').slice(0, 8)}`,
    'schema_version: 2',
    `tags: [busca, persistencia, tag${i % 7}]`,
    'evidence:',
    '  build: true',
  ].join('\n');
  writeFileSync(`bench-corpus/notas/fact/${id}.md`, `---\n${front}\n---\n\n${body}`);
}
console.log(`gerado bench-corpus/notas/fact com ${N} notas`);
