// Mesmo trabalho do path.rs: listar dir, ler ficheiros, dividir frontmatter,
// contar chaves de topo, somar bytes. Sem BigInt (mede o data path, nao o checksum).
import { readdirSync, readFileSync } from 'node:fs';

const dir = './bench-corpus/notas/fact';
const start = performance.now();

let notes = 0, keys = 0, bytes = 0, sum = 0;

for (const name of readdirSync(dir)) {
  if (!name.endsWith('.md')) continue;
  const text = readFileSync(`${dir}/${name}`, 'utf8');
  bytes += Buffer.byteLength(text, 'utf8');
  notes++;

  const lines = text.split('\n');
  if (lines[0] === '---') {
    for (let i = 1; i < lines.length; i++) {
      const line = lines[i];
      if (line === '---') break;
      if (!line.startsWith(' ') && line.includes(':')) keys++;
    }
  }
  const buf = Buffer.from(text, 'utf8');
  for (let i = 0; i < buf.length; i++) sum += buf[i];
}

const ms = performance.now() - start;
console.log(`node  notas=${notes} chaves=${keys} bytes=${bytes} soma=${sum} total=${ms.toFixed(2)}ms por_nota=${((ms * 1000) / notes).toFixed(1)}us`);
