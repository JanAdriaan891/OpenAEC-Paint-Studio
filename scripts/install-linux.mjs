import { readFileSync, writeFileSync, mkdirSync, mkdtempSync, cpSync, existsSync, symlinkSync, renameSync, rmSync, chmodSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { homedir } from 'node:os';
import { execFileSync } from 'node:child_process';

const artifact = process.argv[2];
if (process.platform !== 'linux' || !artifact) throw new Error('Usage: node scripts/install-linux.mjs <Paint-Studio.deb>');
const run = (command, args) => execFileSync(command, args, { encoding: 'utf8' }).trim();
if (run('dpkg-deb', ['--field', resolve(artifact), 'Package']) !== 'open-aec-paint-studio') throw new Error('Not an OpenAEC Paint Studio package');
const data = process.env.XDG_DATA_HOME || join(homedir(), '.local/share');
const state = process.env.XDG_STATE_HOME || join(homedir(), '.local/state');
const base = join(data, 'openaec-paint-studio');
mkdirSync(base, { recursive: true });
const destination = mkdtempSync(join(base, 'build-'));
run('dpkg-deb', ['--extract', resolve(artifact), destination]);
const binary = join(destination, 'usr/bin/openaec-paint-studio');
if (run('ldd', [binary]).includes('not found')) throw new Error('Required Linux libraries are missing');
const link = join(base, 'current.next');
rmSync(link, { force: true }); symlinkSync(destination, link); renameSync(link, join(base, 'current'));
const binDir = join(homedir(), '.local/bin'); mkdirSync(binDir, { recursive: true });
const launcher = join(binDir, 'openaec-paint-studio');
const quote = value => "'" + value.replaceAll("'", "'\\''") + "'";
writeFileSync(launcher, `#!/bin/sh\nexec ${quote(join(base, 'current/usr/bin/openaec-paint-studio'))} "$@"\n`); chmodSync(launcher, 0o755);
cpSync(join(destination, 'usr/share/icons'), join(data, 'icons'), { recursive: true });
const mimeDirectory = join(data, 'mime/packages'); mkdirSync(mimeDirectory, { recursive: true });
cpSync(join(destination, 'usr/share/mime/packages/openaec-paint-studio.xml'), join(mimeDirectory, 'openaec-paint-studio.xml'));
const config = JSON.parse(readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url)));
const mimeTypes = [...new Set([...config.bundle.fileAssociations.map(item => item.mimeType), 'image/x-ms-bmp', 'image/x-icon', 'image/x-photoshop', 'image/heic-sequence', 'image/heif-sequence', 'image/svg+xml-compressed'])];
const backupDirectory = join(state, 'openaec-paint-studio'); mkdirSync(backupDirectory, { recursive: true });
const backupPath = join(backupDirectory, 'file-associations-before-install.json');
if (!existsSync(backupPath)) {
  const defaults = Object.fromEntries(mimeTypes.map(type => [type, run('xdg-mime', ['query', 'default', type])]));
  writeFileSync(backupPath, JSON.stringify(defaults, null, 2) + '\n');
}
const applications = join(data, 'applications'); mkdirSync(applications, { recursive: true });
const desktopId = 'org.openaec.paint-studio.desktop';
const desktopQuote = value => '"' + value.replace(/[\\"`$]/g, '\\$&') + '"';
writeFileSync(join(applications, desktopId), `[Desktop Entry]\nType=Application\nName=OpenAEC Paint Studio\nComment=Afbeeldingen tekenen en bewerken\nExec=${/^[A-Za-z0-9/_.-]+$/.test(launcher) ? launcher : desktopQuote(launcher)} %f\nTryExec=${launcher}\nIcon=openaec-paint-studio\nTerminal=false\nCategories=Graphics;RasterGraphics;\nMimeType=${mimeTypes.join(';')};\nStartupNotify=true\nStartupWMClass=openaec-paint-studio\n`);
// Remove only this app's obsolete project association, preserving other defaults.
const defaultsFile = join(process.env.XDG_CONFIG_HOME || join(homedir(), '.config'), 'mimeapps.list');
if (existsSync(defaultsFile)) {
  const previous = JSON.parse(readFileSync(backupPath, 'utf8'))['application/x-openaec-paint'] || '';
  const text = readFileSync(defaultsFile, 'utf8').replace(/^application\/x-openaec-paint=(.*)$/gm, (line, value) => {
    const entries = value.split(';').filter(Boolean);
    if (!entries.includes(desktopId)) return line;
    const kept = entries.filter(entry => entry !== desktopId);
    const restored = kept.length ? kept.join(';') : previous;
    return restored ? `application/x-openaec-paint=${restored}` : '';
  });
  writeFileSync(defaultsFile, text);
}
run('update-mime-database', [join(data, 'mime')]);
run('update-desktop-database', [applications]);
for (const type of mimeTypes.filter(type => type.startsWith('image/'))) run('xdg-mime', ['default', desktopId, type]);
console.log(`Installed: ${launcher}\nDesktop entry: ${join(applications, desktopId)}\nAssociated ${mimeTypes.length} image/IFCX MIME types (image defaults updated). Previous defaults: ${backupPath}`);
