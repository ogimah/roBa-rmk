"""Invoke fbs via the selected interpreter and always persist its output."""
from pathlib import Path
import subprocess
import sys
import traceback

path = Path.cwd() / "freeze.log"
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
