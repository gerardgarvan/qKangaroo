#!/usr/bin/env python3
"""Verify a q-Kangaroo installation is working correctly.

Usage:
    python check_install.py          # End-user checks
    python check_install.py --dev    # + build-from-source checks
"""

import argparse
import os
import platform
import subprocess
import sys


def supports_color():
    """Check if the terminal supports ANSI color codes."""
    if os.environ.get("NO_COLOR"):
        return False
    if sys.platform == "win32":
        # Windows 10+ supports ANSI via virtual terminal processing
        return os.environ.get("TERM") or os.environ.get("WT_SESSION") or os.environ.get("ANSICON")
    return hasattr(sys.stdout, "isatty") and sys.stdout.isatty()


_use_color = supports_color()


def _pass(msg):
    """Print a PASS line."""
    if _use_color:
        print(f"  \033[32m[PASS]\033[0m {msg}")
    else:
        print(f"  [PASS] {msg}")


def _fail(msg):
    """Print a FAIL line."""
    if _use_color:
        print(f"  \033[31m[FAIL]\033[0m {msg}")
    else:
        print(f"  [FAIL] {msg}")


def check_python_version():
    """Check Python version >= 3.9."""
    ver = sys.version_info
    ver_str = f"{ver.major}.{ver.minor}.{ver.micro}"
    if ver >= (3, 9):
        _pass(f"Python version: {ver_str}")
        return True
    else:
        _fail(f"Python version {ver_str} is too old (need >= 3.9)")
        return False


def check_import():
    """Check that q_kangaroo can be imported."""
    try:
        from q_kangaroo import partition_count, QSession, etaq  # noqa: F401
        _pass("Import q_kangaroo")
        return True
    except ImportError as e:
        _fail(f"Import q_kangaroo: {e}")
        return False


def check_gmp_loading():
    """Check that the native extension loads (requires GMP)."""
    try:
        from q_kangaroo import QSession
        QSession()
        _pass("GMP / native extension loading")
        return True
    except Exception as e:
        _fail(f"GMP / native extension loading: {e}")
        return False


def check_computation():
    """Check that partition_count(50) == 204226."""
    try:
        from q_kangaroo import partition_count
        result = partition_count(50)
        if result == 204226:
            _pass(f"Basic computation: partition_count(50) = {result}")
            return True
        else:
            _fail(f"Basic computation: partition_count(50) = {result} (expected 204226)")
            return False
    except Exception as e:
        _fail(f"Basic computation: {e}")
        return False


def _run_command(cmd):
    """Run a command and return (success, stdout_stripped)."""
    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=30
        )
        if result.returncode == 0:
            return True, result.stdout.strip()
        return False, result.stderr.strip()
    except FileNotFoundError:
        return False, "command not found"
    except subprocess.TimeoutExpired:
        return False, "command timed out"


def check_rust():
    """Check rustc is available."""
    ok, output = _run_command(["rustc", "--version"])
    if ok and "rustc" in output:
        _pass(f"Rust toolchain: {output}")
        return True
    else:
        _fail(f"Rust toolchain: {output}")
        return False


def check_cargo():
    """Check cargo is available."""
    ok, output = _run_command(["cargo", "--version"])
    if ok:
        _pass(f"Cargo: {output}")
        return True
    else:
        _fail(f"Cargo: {output}")
        return False


def check_maturin():
    """Check maturin is available."""
    ok, output = _run_command([sys.executable, "-m", "maturin", "--version"])
    if ok:
        _pass(f"Maturin: {output}")
        return True
    else:
        _fail(f"Maturin: not installed (pip install maturin)")
        return False


def check_gmp_headers():
    """Check that gmp.h can be found."""
    search_paths = []

    # Check C_INCLUDE_PATH env var
    c_include = os.environ.get("C_INCLUDE_PATH", "")
    if c_include:
        search_paths.extend(c_include.split(os.pathsep))

    # Platform-specific common locations
    if sys.platform == "win32" or platform.system() == "Windows":
        search_paths.extend([
            r"C:\mingw64-gcc\mingw64\include",
            r"C:\msys64\mingw64\include",
            r"C:\mingw64\include",
        ])
    else:
        search_paths.extend([
            "/usr/include",
            "/usr/local/include",
            "/usr/include/x86_64-linux-gnu",
        ])

    for path in search_paths:
        gmp_h = os.path.join(path, "gmp.h")
        if os.path.isfile(gmp_h):
            _pass(f"GMP headers: {gmp_h}")
            return True

    _fail("GMP headers: gmp.h not found in standard locations")
    return False


def check_c_compiler():
    """Check that a C compiler (gcc) is available."""
    ok, output = _run_command(["gcc", "--version"])
    if ok:
        # First line of gcc --version output
        first_line = output.split("\n")[0] if output else output
        _pass(f"C compiler: {first_line}")
        return True

    # On Windows, also try cc as fallback
    if sys.platform == "win32" or platform.system() == "Windows":
        ok, output = _run_command(["cc", "--version"])
        if ok:
            first_line = output.split("\n")[0] if output else output
            _pass(f"C compiler: {first_line}")
            return True

    _fail("C compiler: gcc not found")
    return False


def main():
    parser = argparse.ArgumentParser(
        description="Verify q-Kangaroo installation"
    )
    parser.add_argument(
        "--dev", action="store_true",
        help="Run additional build-from-source prerequisite checks"
    )
    args = parser.parse_args()

    results = []

    print("q-Kangaroo Installation Check")
    print("=" * 40)
    print()
    print("End-user checks:")

    results.append(check_python_version())
    results.append(check_import())
    results.append(check_gmp_loading())
    results.append(check_computation())

    if args.dev:
        print()
        print("Developer checks:")
        results.append(check_rust())
        results.append(check_cargo())
        results.append(check_maturin())
        results.append(check_gmp_headers())
        results.append(check_c_compiler())

    passed = sum(results)
    total = len(results)
    print()
    print(f"{passed}/{total} checks passed")

    sys.exit(0 if passed == total else 1)


if __name__ == "__main__":
    main()
