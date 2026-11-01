import { spawn, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
const exe = process.platform === 'win32' ? '.exe' : '';
const webOnly = process.argv.includes('--web-only');
if (
  !webOnly &&
  !process.env.HYPERREALISM_BIN &&
  !existsSync(path.join(root, `target/release/hyperrealism-core${exe}`)) &&
  !existsSync(path.join(root, `target/debug/hyperrealism-core${exe}`))
) {
  console.log('Building the Rust CPU engine…');
  const result = spawnSync(process.platform === 'win32' ? 'cargo.exe' : 'cargo', ['build', '--locked'], {
    cwd: root,
    stdio: 'inherit',
  });
  if (result.status !== 0) {
    console.error('Install Rust from https://rustup.rs, or use npm run dev:web to work on the UI only.');
    process.exit(1);
  }
}
const children = [
  spawn(process.execPath, ['--import', 'tsx', 'server/index.ts'], {
    cwd: root,
    stdio: 'inherit',
    env: process.env,
  }),
  spawn(process.execPath, ['node_modules/vite/bin/vite.js', '--host', '0.0.0.0'], {
    cwd: root,
    stdio: 'inherit',
    env: process.env,
  }),
];
let closing = false;
const close = () => {
  if (closing) return;
  closing = true;
  for (const child of children) child.kill('SIGTERM');
};
for (const child of children) {
  child.on('error', (e) => {
    console.error(e);
    close();
    process.exitCode = 1;
  });
  child.on('exit', (code) => {
    if (!closing && code !== 0) process.exitCode = code || 1;
    close();
  });
}
process.on('SIGINT', close);
process.on('SIGTERM', close);
