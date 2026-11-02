#!/usr/bin/env python3
"""Build once and measure five isolated processes for each fixed input."""
import argparse
import datetime
import hashlib
import json
import os
import platform
import statistics
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--output', default='artifacts/benchmarks')
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
output = root / args.output
output.mkdir(parents=True, exist_ok=True)


def source_hash():
    files = [root / 'Cargo.toml', root / 'Cargo.lock']
    files += sorted((root / 'crates/paint-core').rglob('*.rs'))
    files.append(root / 'crates/paint-core/Cargo.toml')
    digest = hashlib.sha256()
    for path in files:
        digest.update(str(path.relative_to(root)).encode())
        digest.update(path.read_bytes())
    return digest.hexdigest()


def command(*args):
    return subprocess.check_output(args, cwd=root, text=True).strip()


fingerprint = source_hash()
environment = {
    'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'commit': command('git', 'rev-parse', 'HEAD'),
    'working_tree_dirty': bool(command('git', 'status', '--porcelain', '--untracked-files=no')),
    'os': platform.platform(),
    'machine': platform.machine(),
    'cpu': platform.processor(),
    'logical_cpus': os.cpu_count(),
    'rustc': command('rustc', '--version'),
    'source_sha256': fingerprint,
    'build': 'release, locked Cargo dependencies',
    'method': ('Five separate native processes per image on a shared workstation. '
               'Input generation, file read and compilation excluded from phase timings. '
               'Process RSS includes allocations across all phases. Technical timings, '
               'not human task times or competitor comparisons.'),
}
if Path('/proc/cpuinfo').exists():
    environment['cpu_info'] = next((line.split(':', 1)[1].strip()
        for line in Path('/proc/cpuinfo').read_text().splitlines()
        if line.startswith('model name')), None)
if Path('/proc/meminfo').exists():
    environment['memory_total'] = Path('/proc/meminfo').read_text().splitlines()[0]

subprocess.run(['cargo', 'build', '--release', '--locked', '-p', 'paint-core',
                '--example', 'benchmark'], cwd=root, check=True)
binary = root / 'target/release/examples/benchmark'
if os.name == 'nt':
    binary = binary.with_suffix('.exe')
if hasattr(os, 'getloadavg'):
    environment['load_average_before_samples'] = os.getloadavg()
results = []
for width, height in [(1200, 800), (3840, 2160), (6000, 4000)]:
    image = output / f'{width}x{height}.png'
    subprocess.run([str(binary), 'generate', str(image), str(width), str(height)],
                   check=True, timeout=120)
    samples, errors = [], []
    for repeat in range(5):
        try:
            run = subprocess.run([str(binary), str(image)], capture_output=True,
                                 text=True, check=True, timeout=180)
            samples.append(json.loads(run.stdout))
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired,
                json.JSONDecodeError) as error:
            errors.append({'repeat': repeat + 1, 'error': str(error),
                           'stderr': str(getattr(error, 'stderr', None))})
    summary = {}
    if samples:
        for phase in samples[0]['timings_ms']:
            values = [sample['timings_ms'][phase] for sample in samples]
            summary[phase] = {'median_ms': statistics.median(values),
                              'min_ms': min(values), 'max_ms': max(values),
                              'stdev_ms': statistics.pstdev(values)}
    results.append({'input': image.name,
                    'sha256': hashlib.sha256(image.read_bytes()).hexdigest(),
                    'samples': samples, 'errors': errors, 'summary': summary})
    print(image.name, json.dumps(summary), flush=True)
environment['completed_at_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
environment['source_changed_during_run'] = source_hash() != fingerprint
(output / 'results.json').write_text(json.dumps({
    'environment': environment, 'results': results}, indent=2) + '\n')
if environment['source_changed_during_run'] or any(row['errors'] for row in results):
    raise SystemExit('Benchmark source changed or samples failed; inspect results.json')
