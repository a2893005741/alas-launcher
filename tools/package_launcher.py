"""Package the Windows x64 portable launcher."""
from pathlib import Path
import shutil
import zipfile
root = Path(__file__).resolve().parents[1]
package = root / 'package-root/WarshipPilot'
package.mkdir(parents=True, exist_ok=True)
executable = root / 'target/release/warshippilot-launcher.exe'
shutil.copy2(executable, package / 'WarshipPilot.exe')
with zipfile.ZipFile(root / 'WarshipPilot-Windows-x86_64.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
    archive.write(package / 'WarshipPilot.exe', 'WarshipPilot.exe')
