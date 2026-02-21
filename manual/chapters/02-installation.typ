// 02-installation.typ -- Installation (1-2 pages)
#import "../template.typ": *

= Installation <installation>
#index[installation]

q-Kangaroo is distributed as a single static binary with no runtime
dependencies. Pre-built binaries are available for Linux (x86_64) and
Windows (x86_64). A Python API package is also available.

== Pre-built Binaries

Download the latest release from the
#link("https://github.com/q-kangaroo/q-kangaroo/releases")[GitHub Releases]
page. Each release includes:

- `q-kangaroo-linux-x86_64.tar.gz` -- Linux binary
- `q-kangaroo-windows-x86_64.zip` -- Windows binary
- `q-kangaroo-manual.pdf` -- This reference manual

== Linux

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("tar xzf q-kangaroo-linux-x86_64.tar.gz
chmod +x q-kangaroo
sudo mv q-kangaroo /usr/local/bin/", lang: none)
]

Alternatively, place the binary anywhere on your `PATH`.

== Windows

Extract the zip archive. The `q-kangaroo.exe` binary can be run directly
from any directory. Optionally, add the directory containing the executable
to your system `PATH` for convenient access from any command prompt.

== Building from Source
#index[building from source]

Building q-Kangaroo from source requires:

- *Rust 1.85 or later* (install via #link("https://rustup.rs")[rustup])
- *C compiler* (for GMP/MPFR/MPC, which are compiled from source automatically)

Clone the repository and build the release binary:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("git clone https://github.com/q-kangaroo/q-kangaroo.git
cd q-kangaroo
cargo build --release -p qsym-cli", lang: none)
]

The binary is produced at `target/release/q-kangaroo` (or `q-kangaroo.exe`
on Windows). The first build takes several minutes because GMP, MPFR, and
MPC are compiled from source. Subsequent builds are fast due to caching.

== Python API
#index[Python API]

The Python bindings are available as a separate package:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("pip install q-kangaroo", lang: none)
]

The Python API provides the same 97 functions through a `QSession` object.
See the Python documentation for details.

== Verifying Installation

Run `q-kangaroo --version` to confirm the binary is installed correctly:

#repl("--version", "q-kangaroo 0.9.0")

If the REPL launches but you see unexpected behavior, try `q-kangaroo -v`
(verbose mode) to see per-statement timing, which can help diagnose issues.
