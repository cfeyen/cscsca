#!python

import sys
import subprocess

failures = 0

sys.stdout.reconfigure(encoding='utf-8')
sys.stderr.reconfigure(encoding='utf-8')

for cmd in sys.argv[1:]:
    print(f"\x1b[34mRunning Test\x1b[0m: {cmd}")

    result = subprocess.run(
        args = cmd.split(),
        encoding="utf-8",
        capture_output=True,
    )

    if result.returncode != 0:
        failures += 1
        print(f"\x1b[91mTest Failed\x1b[0m:")
        print(result.stdout)
    else:
        print(f"\x1b[32mTest Passed\x1b[0m")

print(f"{failures} failures occured")

exit_code = min(failures, 1)
sys.exit(exit_code)