"""Fail on the first pip error and retain the actual dependency output."""
from pathlib import Path
import subprocess
import sys

commands = [
    [sys.executable, "-m", "pip", "install", "pip==21.3.1", "setuptools==57.5.0",
     "wheel==0.37.1"],
    # The exact alpha specified by vial-kb/cython-hidapi is no longer on
    # PyPI. Build the same version from its official, pinned Git commit.
    # Pure-Python Cython avoids compiling the compiler itself; generated
    # wrapper code and the pinned HIDAPI source remain unchanged.
    [sys.executable, "-m", "pip", "install", "--global-option=--no-cython-compile",
     "git+https://github.com/cython/cython@20f478a778ba6c34ff8ccadb289bf01cfbe90b94#egg=Cython"],
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
