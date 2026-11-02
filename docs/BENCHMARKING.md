# Reproducible task and platform measurements (#7)

This protocol separates technical measurements, automated functional acceptance and human usability tasks. It makes no speed or ease-of-use claim against a competitor without a like-for-like measurement.

## Run the technical benchmark

With the locked Rust toolchain/dependencies:

```sh
python3 scripts/benchmark.py
```

The script builds the release benchmark once, creates deterministic RGBA PNG inputs at 1200 × 800, 3840 × 2160 and 6000 × 4000, then launches five isolated native processes per input. `artifacts/benchmarks/results.json` contains every sample, errors, median/min/max/standard deviation, input SHA256, exact source commit/dirty flag, Rust version, OS and CPU. Preserve that file with any published table. File reading and fixture generation are outside the phase timings; PNG decoding, scaling and encoding are inside their relevant phases. Peak process RSS is available on Linux and covers all phases, not an individual operation. Record other platforms as unavailable unless separately instrumented.

Phases: image import, render, PNG export, automated crop + three labels + alignment + shape + PNG export, editable project write, project read including validation/render, pixel-drag preparation/cached preview, rectangular pixel move with flattening, and undo/redo. The large input deliberately reports both original and actual canvas sizes. It must not receive an unfair speed score against another editor operating at full 6000 × 4000 resolution.

Undo/redo timings cover document snapshot swaps, excluding repaint and draft persistence. Cached pixel-preview timings exclude conversion into the UI image handle and display rendering. These are warm-cache, sequential technical samples, not cold-launch times, human task times, first-use experience or evidence that Paint is faster than another editor. Memory may be influenced by allocator high-water behavior. Repeat only when changes or environmental interference justify it.

## Shared acceptance tasks

| Task | Input and success condition | Record |
| --- | --- | --- |
| Screenshot annotate | Paste 1200 × 800 screenshot, crop, add text and rectangle, export PNG | Human time, actions, mistakes, output alpha/dimensions; permission prompts separately |
| Continue editing | Save, close, reopen multiline text + two shapes + image with transforms/opacity | Independent edits, visual match and error/cancel behavior |
| Pixel selection | Move/delete a rectangle from a photo; undo/redo | Source pixels, destination clipping, zoom/pan, flattening warning and restored objects |
| Composition | Choose only three labels and align; explore stacking | Selection effort; mark hiding/locking/ordering unavailable until implemented |
| Large import/export | Use all three fixed input sizes | Original and result dimensions, confirmation/cancel, technical samples and errors |
| Recovery/offline | Restore draft after reload; start offline both cold and after a visit | Desktop and browser separately; distinguish a disconnected server from offline PWA support |
| Automation | Apply the same fixed labels with API/MCP or supported scripting | Setup, edits/corrections, output dimensions/quality; do not time an unavailable automation feature |

For human tasks use a fixed target result, exact product version, OS/browser/hardware and date. Count user actions consistently (click, key chord, drag and text entry separately). Separate first-time users from developers familiar with this app. Run five technical repetitions and keep spread/errors. Never invent timings for missing functionality. Background removal requires a fixed image-quality target before comparing it.

## Comparison and platform reporting

The results document records installed/available products and platforms. Microsoft Paint/Paint.NET need a Windows host; macOS Finder/file association acceptance needs macOS. Pinta, JS Paint and Photopea must be measured using a fixed version or dated web build and equivalent output. A source inspection or vendor feature claim is not a benchmark. Unavailable measurements are explicitly marked and do not turn into performance claims.

For the build's functional baseline use `cargo test --workspace --locked`, `tests/web-smoke.mjs`, `tests/web-workflows.mjs` and `tests/native-smoke.py`. The browser tests use isolated profiles and never clear or overwrite the user's preview draft. Clipboard permissions and OS clipboard persistence need separate platform acceptance.

## Image clipboard and file acceptance matrix

| Platform | File behavior to accept | Clipboard behavior to accept | Evidence / remaining acceptance |
| --- | --- | --- | --- |
| Windows | Native image/project filters, Cancel, atomic replacement and project reopen | Ctrl+C/V; Ctrl+X in Pixels mode; screenshot DIB/bitmap, dimensions and alpha where supported | Core/state tests and platform CI; physical native dialogs and cross-app clipboard pending |
| macOS Intel/Apple Silicon | Native dialogs, Cmd+S / Cmd+Shift+O, Finder open and atomic replacement | Cmd+C/V/X; native NSImage exchange, alpha and empty clipboard | Architecture builds/process smoke in CI; physical Finder/dialog/clipboard acceptance pending |
| Linux X11/Wayland | Portal/native filters, file argument/drop, cancellation and atomic replacement | Ctrl+C/V/X; PNG exchange; owner persists while app runs; Wayland data-control availability | Core/state tests; CI window startup; physical desktop/portal/clipboard acceptance pending |
| Browser Chromium | Image-only Open, separate JSON project Open, cancellable Download handoff; draft reload | PNG Clipboard API with permission on HTTPS/localhost; text fields retain text behavior | Actual isolated-profile workflow tests passed on Chromium 156.0.8078.4/Linux; other browser engines pending |

The [Arboard adapter documentation](https://docs.rs/arboard/3.7.0/arboard/struct.Clipboard.html) describes native clipboard transport, not proof that every other application preserves alpha. A browser download handoff cannot establish completion on disk; cancellation of our dialog is tested independently. Native saves return success only after the complete file has replaced its target.
