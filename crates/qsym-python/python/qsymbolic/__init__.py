"""Q-Symbolic: Symbolic computation for q-series."""

import os
import sys

# On Windows, Python 3.8+ requires explicit DLL directory registration
# for native extensions that depend on shared libraries (e.g., GMP via MinGW).
if sys.platform == "win32":
    _mingw_dir = os.environ.get("MINGW_BIN", r"C:\mingw64-gcc\mingw64\bin")
    if os.path.isdir(_mingw_dir):
        os.add_dll_directory(_mingw_dir)

from qsymbolic._qsymbolic import QSession, QExpr, version

__version__ = version()


def symbols(names: str, session=None):
    """Create symbols from a whitespace-separated string.

    If no session is provided, creates a new one.
    Returns a tuple of QExpr for multiple names, or a single QExpr for one name.

    Examples:
        q, a = symbols("q a")
        x = symbols("x")
    """
    if session is None:
        session = QSession()
    result = session.symbols(names)
    if len(result) == 1:
        return result[0]
    return tuple(result)


__all__ = ["QSession", "QExpr", "symbols", "__version__"]
