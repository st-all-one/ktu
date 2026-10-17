const bits = (v) => new DataView(new Float64Array([v]).buffer).getBigUint64(0, true).toString(16).padStart(16, '0');
const out = [];
const xb = [1e-8, 0.001, 0.5, 1.0, 10.0, 777.6666666666666, 1e6, 1e15];

const pad = (v, w) => {
  const s = String(v);
  return s.length >= w ? s : ' '.repeat(w - s.length) + s;
};

for (const x of xb) out.push(`log1p ${pad(x, 22)} ${bits(Math.log1p(x))}`);
for (const x of xb) out.push(`log   ${pad(x, 22)} ${bits(Math.log(x))}`);
for (const x of xb) out.push(`sqrt  ${pad(x, 22)} ${bits(Math.sqrt(x))}`);
for (const x of xb) out.push(`exp   ${pad(x, 22)} ${bits(Math.exp(Math.min(x, 20.0)))}`);
for (const [b, e] of [[1.2, 3.7], [0.5, 0.019178082191780823], [2.0, 0.5], [10.0, 0.3], [0.85, 30.0]]) {
  out.push(`pow   ${pad(b, 6)}^${pad(e, 22)} ${bits(Math.pow(b, e))}`);
}

// Formatação número -> texto (o emissor TOON depende daqui)
const nums = [0.7, 1.0, 0.0, -0.0, 1e-7, 1e-6, 1.5e-5, 1e21, 1e20, 123456789012345680000.0, 0.1 + 0.2, 3.0, 1e6, 1e-10];
for (const n of nums) {
  let dbg;
  if (Object.is(n, -0)) dbg = '-0.0';
  else if (Number.isInteger(n) && Math.abs(n) < 1e16) dbg = n.toFixed(1);
  else dbg = String(n);
  out.push(`fmt   ${pad(dbg, 26)} => ${String(n)}`);
}

console.log(out.join('\n'));
