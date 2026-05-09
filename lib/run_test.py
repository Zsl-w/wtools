python3 << 'PYEOF'
import subprocess
import time
import sys

# Run the executable
proc = subprocess.Popen(
    [r"e:\Claw\wtools\lib\build\windows\x64\runner\Debug\lib.exe"],
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE
)

# Wait up to 10 seconds
try:
    stdout, stderr = proc.communicate(timeout=10)
    print("STDOUT:", stdout.decode('utf-8', errors='replace'))
    print("STDERR:", stderr.decode('utf-8', errors='replace'))
    print("EXIT CODE:", proc.returncode)
except subprocess.TimeoutExpired:
    print("Process still running after 10 seconds, PID:", proc.pid)
    proc.kill()
    print("Killed")
PYEOF
