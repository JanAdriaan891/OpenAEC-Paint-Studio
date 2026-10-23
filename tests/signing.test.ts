import { test } from 'node:test';
import assert from 'node:assert/strict';
import { generateKeyPairSync } from 'node:crypto';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
test('release signatures reject modified artifacts and untrusted keys', () => {
  const dir = mkdtempSync(join(tmpdir(), 'paint-sign-'));
  try {
    const key = generateKeyPairSync('ed25519'), other = generateKeyPairSync('ed25519');
    const privatePath = join(dir, 'private.pem'), publicPath = join(dir, 'public.pem'), wrongPath = join(dir, 'wrong.pem'), artifact = join(dir, 'app.zip');
    writeFileSync(privatePath, key.privateKey.export({ type: 'pkcs8', format: 'pem' }), { mode: 0o600 });
    writeFileSync(publicPath, key.publicKey.export({ type: 'spki', format: 'pem' })); writeFileSync(wrongPath, other.publicKey.export({ type: 'spki', format: 'pem' }));
    writeFileSync(artifact, 'build');
    const signed = spawnSync(process.execPath, ['scripts/sign.mjs', artifact], { env: { ...process.env, PAINT_SIGNING_KEY_FILE: privatePath } }); assert.equal(signed.status, 0, signed.stderr.toString());
    assert.equal(spawnSync(process.execPath, ['scripts/verify.mjs', artifact, publicPath]).status, 0);
    assert.notEqual(spawnSync(process.execPath, ['scripts/verify.mjs', artifact, wrongPath]).status, 0);
    writeFileSync(artifact, 'tampered build'); assert.notEqual(spawnSync(process.execPath, ['scripts/verify.mjs', artifact, publicPath]).status, 0);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
