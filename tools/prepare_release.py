"""Generate a complete manifest after executable and component verification."""
import argparse
import hashlib
import json
import shutil
import tomllib
from pathlib import Path
from merge_stable_manifest import merge


def prepare(root, output):
    version = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
    config = json.loads((root / 'tauri.conf.json').read_text(encoding='utf-8'))
    if config['version'] != version:
        raise ValueError('Cargo 与 Tauri 版本不一致')
    output.mkdir(parents=True, exist_ok=True)
    exe = output / 'WarshipPilot.exe'
    shutil.copy2(root / 'target/release/warshippilot-launcher.exe', exe)
    direct = f'https://github.com/a2893005741/alas-launcher/releases/download/v{version}/WarshipPilot.exe'
    launcher = {'version': version, 'platforms': {'windows-x86_64': {
        'url': 'https://ghfast.top/' + direct, 'github_url': direct,
        'cloudflare_url': 'https://gh-proxy.com/' + direct,
        'sha256': hashlib.sha256(exe.read_bytes()).hexdigest()}}}
    runtime = json.loads((output / 'windows-x86_64.json').read_text(encoding='utf-8'))
    result = merge({}, launcher=launcher, runtime=runtime)
    (output / 'stable.json').write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    prepare(Path(__file__).resolve().parents[1], args.output.resolve())
