Installation
============

Quick Install (pip)
-------------------

Pre-built wheels are available for Linux (x86_64, manylinux) and Windows
(x86_64).

.. note::

   GMP (the GNU Multiple Precision library) is bundled in the wheels -- no
   system-level dependencies are required.

**Requirements:** Python 3.9 or later.

.. code-block:: bash

   pip install q-kangaroo

.. tip::

   Verify the installation:

   .. code-block:: bash

      python -c "from q_kangaroo import partition_count; assert partition_count(50) == 204226; print('q-Kangaroo is working!')"

Build from Source
-----------------

Prerequisites (all platforms)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- Git
- Python 3.9+
- Rust stable toolchain (install via `rustup <https://rustup.rs>`_)
- `maturin <https://github.com/PyO3/maturin>`_ (``pip install maturin``)
- GMP (GNU Multiple Precision) development library -- the Rust crate
  ``gmp-mpfr-sys`` links against system GMP via the ``use-system-libs`` feature

Linux (Ubuntu / Debian)
^^^^^^^^^^^^^^^^^^^^^^^

#. Install system dependencies:

   .. code-block:: bash

      sudo apt-get update && sudo apt-get install -y build-essential libgmp-dev pkg-config

   For Fedora / RHEL / CentOS:

   .. code-block:: bash

      sudo dnf install gcc gmp-devel pkgconfig

#. Install Rust:

   .. code-block:: bash

      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
      source "$HOME/.cargo/env"

#. Clone the repository:

   .. code-block:: bash

      git clone https://github.com/OWNER/q-kangaroo.git
      cd q-kangaroo

#. Create a virtual environment:

   .. code-block:: bash

      python3 -m venv .venv
      source .venv/bin/activate

#. Install maturin:

   .. code-block:: bash

      pip install maturin

#. Build the extension:

   .. code-block:: bash

      cd crates/qsym-python
      PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --release

#. Verify the build:

   .. code-block:: bash

      cd ../..
      python -c "from q_kangaroo import partition_count; assert partition_count(50) == 204226; print('Build successful!')"

#. Run tests:

   .. code-block:: bash

      pip install pytest
      pytest crates/qsym-python/tests/ -v

Cygwin / Windows (MinGW)
^^^^^^^^^^^^^^^^^^^^^^^^

This section covers building on Windows using Cygwin with a MinGW-w64 GMP
installation.

#. **Install Cygwin** from https://cygwin.com with packages: ``gcc-core``,
   ``make``, ``pkg-config``, ``git``, ``python3`` (or use a standalone Windows
   Python).

#. **Install MinGW GMP.** Download pre-built MSYS2 MinGW-w64 packages or use
   an existing MinGW-w64 installation. The project expects GMP at
   ``C:\mingw64-gcc\mingw64\`` by default. You need these files:

   - ``bin/libgmp-10.dll``
   - ``lib/libgmp.a`` (or ``libgmp.dll.a``)
   - ``include/gmp.h``

#. **Install Rust via rustup.** The project requires the GNU target -- **not**
   MSVC.

   .. warning::

      You **must** use the ``x86_64-pc-windows-gnu`` Rust target. The MSVC
      target will not work because the project links against MinGW-built GMP.

   .. code-block:: bash

      rustup default stable-x86_64-pc-windows-gnu

   If Rust is already installed with the wrong target:

   .. code-block:: bash

      rustup target add x86_64-pc-windows-gnu
      rustup default stable-x86_64-pc-windows-gnu

#. **Set PATH in Cygwin shell.** Run this in every new shell, or add it to
   ``~/.bashrc``:

   .. code-block:: bash

      export PATH="/c/mingw64-gcc/mingw64/bin:/c/cygwin64/bin:/c/Users/$USER/.cargo/bin:$PATH"

#. **Set environment variables for GMP discovery:**

   .. code-block:: bash

      export DEP_GMP_LIB_DIR="C:/mingw64-gcc/mingw64/lib"
      export DEP_GMP_INCLUDE_DIR="C:/mingw64-gcc/mingw64/include"
      export LIBRARY_PATH="C:/mingw64-gcc/mingw64/lib"
      export C_INCLUDE_PATH="C:/mingw64-gcc/mingw64/include"

#. **Clone and build:**

   .. code-block:: bash

      git clone https://github.com/OWNER/q-kangaroo.git
      cd q-kangaroo/crates/qsym-python
      python -m venv .venv
      source .venv/Scripts/activate  # Note: Scripts on Windows, not bin
      pip install maturin
      PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --release

#. **Verify the build:**

   .. code-block:: bash

      cd ../..
      python -c "from q_kangaroo import partition_count; assert partition_count(50) == 204226; print('Build successful!')"

#. **DLL loading at import time.** When you ``import q_kangaroo``, the package
   looks for ``libgmp-10.dll`` in three places (in order):

   #. Bundled in the package directory (used by installed wheels)
   #. Path from the ``MINGW_BIN`` environment variable
   #. Hardcoded fallback: ``C:\mingw64-gcc\mingw64\bin``

   For development builds, ensure MinGW ``bin/`` is accessible via one of these
   paths.

Troubleshooting
---------------

GMP not found during build
^^^^^^^^^^^^^^^^^^^^^^^^^^

**Symptom:**

.. code-block:: text

   error: failed to run custom build command for gmp-mpfr-sys

or:

.. code-block:: text

   ld: cannot find -lgmp

**Cause:** GMP development headers and libraries are not installed or not in
the linker search path.

**Fix (Linux):**

.. code-block:: bash

   sudo apt-get install libgmp-dev

Or for Fedora/RHEL: ``sudo dnf install gmp-devel``

**Fix (Windows):** Set the ``LIBRARY_PATH`` and ``C_INCLUDE_PATH`` environment
variables to point to your MinGW GMP installation (see the
`Cygwin / Windows (MinGW)`_ build section above, step 5).

Wrong Rust target on Windows
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Symptom:**

.. code-block:: text

   error: linker `link.exe` not found

or linker errors mentioning MSVC or ``VCRUNTIME``.

**Cause:** Rust is using the MSVC toolchain instead of GNU.

**Fix:**

.. code-block:: bash

   rustup default stable-x86_64-pc-windows-gnu

The project requires the GNU target because it links against MinGW-built GMP.

cargo or rustc not found in Cygwin
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Symptom:**

.. code-block:: text

   cargo: command not found

or:

.. code-block:: text

   rustc: command not found

**Cause:** The Cargo bin directory is not in the Cygwin PATH.

**Fix:**

.. code-block:: bash

   export PATH="/c/Users/$USER/.cargo/bin:$PATH"

Add this line to ``~/.bashrc`` for persistence.

DLL loading error at import time
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Symptom:**

.. code-block:: text

   ImportError: DLL load failed while importing _q_kangaroo

or:

.. code-block:: text

   libgmp-10.dll not found

**Cause:** Python cannot find the GMP shared library at runtime.

**Fix (pick one):**

- Set ``MINGW_BIN`` environment variable to your MinGW bin directory
  (e.g., ``set MINGW_BIN=C:\mingw64-gcc\mingw64\bin``)
- Ensure ``C:\mingw64-gcc\mingw64\bin`` exists and contains ``libgmp-10.dll``
- Copy ``libgmp-10.dll`` into the ``q_kangaroo`` package directory
  (e.g., ``.venv/Lib/site-packages/q_kangaroo/``)

Python version too old
^^^^^^^^^^^^^^^^^^^^^^

**Symptom:**

.. code-block:: text

   ERROR: q-kangaroo requires Python >=3.9

or syntax errors on import.

**Cause:** Python version is 3.8 or older.

**Fix:** Install Python 3.9 or later from https://python.org.

maturin not installed or build fails
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Symptom:**

.. code-block:: text

   maturin: command not found

or:

.. code-block:: text

   ModuleNotFoundError: No module named 'maturin'

**Cause:** maturin is not installed in the current Python environment.

**Fix:**

.. code-block:: bash

   pip install maturin

maturin must be installed in the same virtual environment where you want to
build q-kangaroo.
