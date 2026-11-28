"""Fail on the first pip error and retain the actual dependency output."""
from pathlib import Path
import subprocess
import sys

commands = [
    [sys.executable, "-m", "pip", "install", "pip==21.3.1", "setuptools==59.6.0",
     "wheel==0.37.1", "Cython==0.29.37"],
    [sys.executable, "-m", "pip", "install", "-r", "requirements.txt"],
    [sys.executable, "-m", "pip", "freeze"],
]
log = Path("dependency-install.log")
log.write_text("", encoding="utf-8")
for command in commands:
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    output = result.stdout.decode("utf-8", errors="replace")
    with log.open("a", encoding="utf-8") as stream:
        stream.write("\nCOMMAND: " + " ".join(command) + "\n" + output)
    print(output)
    if result.returncode:
        sys.exit(result.returncode)
Path("diagnostic-dependencies.txt").write_text(output, encoding="utf-8")
