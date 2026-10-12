import { readdir, readFile, access } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const required = ['README.md', 'LICENSE', 'LICENSE_NOTICES.md', 'CONTRIBUTING.md', 'SECURITY.md', 'CHANGELOG.md', 'docs/README.md'];
const failures = [];
for (const name of required) {
  try { await access(path.join(root, name)); } catch { failures.push(`Ausente: ${name}`); }
}
async function markdownFiles(dir) {
  const files = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    if (['.git', '.tools', 'artifacts', 'node_modules', 'target', 'dist', 'gen'].includes(entry.name)) continue;
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...await markdownFiles(file));
    else if (entry.name.endsWith('.md')) files.push(file);
  }
  return files;
}
const files = await markdownFiles(root);
let links = 0;
for (const file of files) {
  const text = await readFile(file, 'utf8');
  for (const match of text.matchAll(/!?\[[^\]]*\]\(([^)]+)\)/g)) {
    const target = match[1].split(/\s+["']/)[0];
    if (/^(https?:|mailto:|#)/i.test(target)) continue;
    const pathname = decodeURIComponent(target.split('#')[0]);
    if (!pathname) continue;
    links++;
    try { await access(path.resolve(path.dirname(file), pathname)); }
    catch { failures.push(`${path.relative(root, file)}: ${target}`); }
  }
}
if (failures.length) {
  console.error('Falhas na documentação:\n' + failures.join('\n'));
  process.exit(1);
}
console.log(`Documentação válida: ${files.length} arquivos Markdown, ${links} links locais.`);
