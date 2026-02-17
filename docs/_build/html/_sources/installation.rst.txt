Installation
============

From PyPI
---------

The simplest way to install q-Kangaroo is via pip:

.. code-block:: bash

   pip install q-kangaroo

Pre-built wheels are available for:

- **Linux** (x86_64, manylinux)
- **Windows** (x86_64)

GMP (the GNU Multiple Precision library) is bundled in the wheels, so no
system-level dependencies are required.

**Requirements:**

- Python 3.9 or later

From Source
-----------

To build from source, you need Rust and the GMP development library:

1. Clone the repository:

   .. code-block:: bash

      git clone https://github.com/OWNER/q-kangaroo.git
      cd q-kangaroo

2. Install `maturin <https://www.maturin.rs/>`_ (the Rust-Python build tool):

   .. code-block:: bash

      pip install maturin

3. Build and install in development mode:

   .. code-block:: bash

      cd crates/qsym-python
      PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --release

   On Linux, ensure GMP is installed (``sudo apt-get install libgmp-dev``
   or equivalent). On Windows, MinGW with GMP is required.

Verify Installation
-------------------

After installing, verify everything works:

.. code-block:: python

   from q_kangaroo import QSession, partition_count
   assert partition_count(5) == 7
   print("q-Kangaroo installed successfully!")

Or from the command line:

.. code-block:: bash

   python -c "from q_kangaroo import partition_count; print(partition_count(5))"
   # Expected output: 7
