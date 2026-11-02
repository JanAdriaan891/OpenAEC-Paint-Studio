#!/usr/bin/env python3
"""Fingerprint Wasm and its matching generated loader for reliable upgrades."""
import hashlib
import sys
from pathlib import Path

output = Path(sys.argv[1])
package = output / 'pkg'
wasm = package / 'openaec_paint_studio_bg.wasm'
loader = package / 'openaec_paint_studio.js'
digest = hashlib.sha256(wasm.read_bytes()).hexdigest()[:16]
wasm_name = f'openaec_paint_studio_bg.{digest}.wasm'
loader_text = loader.read_text().replace('openaec_paint_studio_bg.wasm', wasm_name)
loader_digest = hashlib.sha256(loader_text.encode()).hexdigest()[:16]
loader_name = f'openaec_paint_studio.{loader_digest}.js'
wasm.rename(package / wasm_name)
(package / loader_name).write_text(loader_text)
loader.unlink()
index = output / 'index.html'
index.write_text(index.read_text().replace('./pkg/openaec_paint_studio.js', f'./pkg/{loader_name}'))
