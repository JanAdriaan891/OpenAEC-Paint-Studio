import { createPrivateKey, createPublicKey, sign, createHash } from 'node:crypto';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { basename } from 'node:path';
const [artifact] = process.argv.slice(2);
if (!artifact || !process.env.PAINT_SIGNING_KEY_FILE) throw new Error('Usage: PAINT_SIGNING_KEY_FILE=/private/release.pem pnpm sign <artifact>');
// An externally supplied Ed25519 release key. Never generate or commit a private key here.
const key = createPrivateKey(readFileSync(process.env.PAINT_SIGNING_KEY_FILE));
if (key.asymmetricKeyType !== 'ed25519') throw new Error('Use an Ed25519 private key');
if (!existsSync(artifact)) throw new Error('Artifact not found');
const bytes = readFileSync(artifact);
const manifest = Buffer.from(JSON.stringify({ schema: 1, file: basename(artifact), bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') }, null, 2) + '\n');
writeFileSync(`${artifact}.manifest.json`, manifest);
writeFileSync(`${artifact}.sig`, sign(null, manifest, key).toString('base64') + '\n');
writeFileSync(`${artifact}.pub.pem`, createPublicKey(key).export({ type: 'spki', format: 'pem' }));
console.log(`Signed ${basename(artifact)}. Verify against your independently trusted public key.`);
