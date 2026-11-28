"""Invoke fbs via the selected interpreter and always persist its output."""
from pathlib import Path
import os
import subprocess
import sys
import traceback

path = Path.cwd() / "freeze.log"
# fbs copies physical UCRT shim DLLs into its bundle. Windows 2025 provides
# these through API sets, so use the installed Windows SDK's x64 redist copy.
sdk = Path(os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")) / "Windows Kits/10/Redist"
for candidate in sorted(sdk.glob("**/ucrt/DLLs/x64"), reverse=True):
    if (candidate / "api-ms-win-crt-multibyte-l1-1-0.dll").exists():
        os.environ["PATH"] += os.pathsep + str(candidate)
        break
prefix = "Python: {}\nVersion: {}\n".format(sys.executable, sys.version)
path.write_text(prefix, encoding="utf-8")
try:
    result = subprocess.run([sys.executable, "-m", "fbs", "freeze"],
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    output = result.stdout.decode("utf-8", errors="replace")
    path.write_text(prefix + output, encoding="utf-8")
    print(output)
    sys.exit(result.returncode)
except Exception:
    output = traceback.format_exc()
    path.write_text(prefix + output, encoding="utf-8")
    print(output)
    sys.exit(1)
