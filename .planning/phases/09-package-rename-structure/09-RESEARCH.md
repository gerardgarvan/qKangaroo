# Phase 9: Package Rename & Structure - Research

**Researched:** 2026-02-14
**Domain:** PyO3/maturin package renaming, Cargo workspace naming, Python import mechanics
**Confidence:** HIGH

## Summary

Phase 9 is a focused renaming operation: change the Python-facing package name from `qsymbolic` to `q_kangaroo` and the native extension module from `_qsymbolic` to `_q_kangaroo`. The Rust crate names (`qsym-core`, `qsym-python`) remain unchanged -- only the Python-visible names change. This is confirmed by the requirements (REN-01 through REN-05) which exclusively reference `qsymbolic` as the old name to eliminate.

The rename touches exactly 5 source files plus 1 directory rename, plus updates to PROJECT.md. The critical technical constraint is the "triple-match" rule: the `[lib] name` in Cargo.toml, the `module-name` in pyproject.toml, and the `#[pymodule]` function name in lib.rs must all agree on `_q_kangaroo`. Getting any one of these wrong produces a confusing ImportError at runtime even though the build succeeds.

**Primary recommendation:** Perform the rename as an atomic operation across all 5 files + directory, then immediately verify with `cargo test` and `maturin develop && pytest`. Do NOT rename the Rust crate directories or crate names -- the requirements only cover Python-facing names.

## Standard Stack

### Core (No New Dependencies)

This phase introduces zero new dependencies. It is purely a renaming/refactoring operation on existing files.

| Tool | Version | Purpose | Why Used |
|------|---------|---------|----------|
| maturin | >=1.0,<2.0 | Builds Rust cdylib into Python wheel | Already in use; `module-name` config drives output naming |
| PyO3 | 0.23 | Rust-to-Python bridge | Already in use; `#[pymodule]` macro name must match |
| cargo | 1.85.0 | Rust build system | Already in use; `[lib] name` field controls cdylib output |

### No Alternatives Needed

This is not a technology-choice phase. Every tool is already in place. The only "choices" are about naming conventions, which are already decided by the requirements.

## Architecture Patterns

### Current Naming Architecture (BEFORE rename)

```
crates/qsym-python/
  Cargo.toml          -> [lib] name = "_qsymbolic"
  pyproject.toml      -> name = "qsymbolic", module-name = "qsymbolic._qsymbolic"
  src/lib.rs          -> fn _qsymbolic(m: &Bound<'_, PyModule>) -> PyResult<()>
  python/qsymbolic/   -> directory name
    __init__.py        -> from qsymbolic._qsymbolic import ...
  tests/
    test_integration.py -> from qsymbolic import ...
```

### Target Naming Architecture (AFTER rename)

```
crates/qsym-python/
  Cargo.toml          -> [lib] name = "_q_kangaroo"
  pyproject.toml      -> name = "q-kangaroo", module-name = "q_kangaroo._q_kangaroo"
  src/lib.rs          -> fn _q_kangaroo(m: &Bound<'_, PyModule>) -> PyResult<()>
  python/q_kangaroo/  -> directory name (RENAMED from qsymbolic/)
    __init__.py        -> from q_kangaroo._q_kangaroo import ...
  tests/
    test_integration.py -> from q_kangaroo import ...
```

### What Does NOT Change

These Rust-internal names remain unchanged:

| Item | Current Value | Stays Same? |
|------|---------------|-------------|
| Workspace root Cargo.toml `members` | `["crates/qsym-core", "crates/qsym-python"]` | YES -- these are directory paths |
| qsym-core `[package] name` | `"qsym-core"` | YES |
| qsym-python `[package] name` | `"qsym-python"` | YES |
| qsym-python Cargo.toml `[dependencies] qsym-core` | `{ path = "../qsym-core" }` | YES |
| All `use qsym_core::...` statements in Rust source | 106 occurrences across 22 test files + 24 in 6 src files | YES |
| Directory names `crates/qsym-core/`, `crates/qsym-python/` | Filesystem paths | YES |
| `.cargo/config.toml` | GMP build settings | YES |

**Rationale:** In Cargo, `members` references directory paths, not package names. The `[package] name` can differ from the directory name. Since the requirements only say "no references to `qsymbolic`" (not `qsym-core` or `qsym-python`), and since `qsym-core`/`qsym-python` are internal Rust crate names never exposed to Python users, they are out of scope.

### The Triple-Match Rule (CRITICAL)

For maturin to produce a working Python extension, three identifiers MUST match:

```
1. Cargo.toml [lib] name     = "_q_kangaroo"
                                    ^^^^^^^^^^^^^
2. pyproject.toml module-name = "q_kangaroo._q_kangaroo"
                                             ^^^^^^^^^^^^^
3. lib.rs #[pymodule] fn        _q_kangaroo(...)
                                ^^^^^^^^^^^^^
```

The last component of `module-name` MUST exactly equal both the `[lib] name` and the `#[pymodule]` function name. Misalignment causes `ImportError: dynamic module does not define module export function (PyInit__q_kangaroo)` at runtime, even though `maturin develop` succeeds.

### Pattern: PyPI Name vs Import Name

Per PEP 423 / PEP 503, the PyPI package name uses hyphens (`q-kangaroo`) but the Python import name uses underscores (`q_kangaroo`). pip normalizes package names (hyphens to underscores) for installation, so `pip install q-kangaroo` installs the `q_kangaroo` importable package.

```
PyPI name:    q-kangaroo     (used in: pip install q-kangaroo, pyproject.toml [project] name)
Import name:  q_kangaroo     (used in: import q_kangaroo, directory name, module-name prefix)
Native module: _q_kangaroo   (used in: [lib] name, #[pymodule], import q_kangaroo._q_kangaroo)
```

### Anti-Patterns to Avoid

- **Partial rename:** Updating 2 of 3 triple-match locations. Always change all 3 atomically.
- **Renaming Rust crate directories:** Unnecessary complexity. Requirements only target `qsymbolic`.
- **Renaming Rust crate package names:** Would require changing 106+ `use qsym_core::` lines across 22 test files. Not required by any REN requirement.
- **Forgetting the directory rename:** The Python package directory `python/qsymbolic/` must become `python/q_kangaroo/`. This is the most easily forgotten step since it is a filesystem operation, not a text edit.
- **Leaving __pycache__ behind:** After renaming the directory, stale `__pycache__` and `.pyd` files under the old name can cause confusing import behavior. Delete all compiled artifacts before testing.
- **Forgetting PROJECT.md:** The top-level PROJECT.md contains 4 references to `qsymbolic` that should be updated to match the new name.

## Comprehensive File Change List

### Files Requiring Text Edits (5 files)

| File | Changes Needed | Occurrences |
|------|---------------|-------------|
| `crates/qsym-python/Cargo.toml` | `name = "_qsymbolic"` -> `name = "_q_kangaroo"` | 1 |
| `crates/qsym-python/pyproject.toml` | `name = "qsymbolic"` -> `name = "q-kangaroo"`, `module-name = "qsymbolic._qsymbolic"` -> `module-name = "q_kangaroo._q_kangaroo"` | 2 |
| `crates/qsym-python/src/lib.rs` | `fn _qsymbolic(` -> `fn _q_kangaroo(` | 1 |
| `crates/qsym-python/python/q_kangaroo/__init__.py` | `from qsymbolic._qsymbolic import` -> `from q_kangaroo._q_kangaroo import` (2 import lines), docstring | 2-3 |
| `crates/qsym-python/tests/test_integration.py` | `from qsymbolic import` -> `from q_kangaroo import` | 9 |

### Filesystem Operations (1 directory rename + cleanup)

| Operation | From | To |
|-----------|------|-----|
| Rename directory | `crates/qsym-python/python/qsymbolic/` | `crates/qsym-python/python/q_kangaroo/` |
| Delete stale artifacts | `crates/qsym-python/python/qsymbolic/__pycache__/` | (remove) |
| Delete stale .pyd | `crates/qsym-python/python/qsymbolic/_qsymbolic.cp314-win_amd64.pyd` | (remove; maturin will rebuild) |

### Optional: Documentation Updates

| File | Changes |
|------|---------|
| `PROJECT.md` | 4 references to `qsymbolic` in code examples and architecture diagram |

Note: `.planning/` files (PLAN.md, SUMMARY.md, RESEARCH.md, etc.) are historical documentation of past phases and should NOT be updated. The success criterion "No references to the old name `qsymbolic` remain in source files, configs, or test code" applies to *source files, configs, and test code* -- not planning artifacts.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Finding all occurrences | Manual file-by-file search | `grep -r "qsymbolic" --include="*.py" --include="*.rs" --include="*.toml"` | Ensures no reference is missed |
| Directory rename on Windows/Cygwin | Complex mv + git tracking | `git mv crates/qsym-python/python/qsymbolic crates/qsym-python/python/q_kangaroo` | Preserves git history |
| Verifying the triple-match | Manual inspection | Build + import test: `maturin develop && python -c "import q_kangaroo"` | Runtime is the only reliable check |

## Common Pitfalls

### Pitfall 1: Triple-Match Mismatch (ImportError)

**What goes wrong:** After renaming, `import q_kangaroo` fails with `ImportError: dynamic module does not define module export function (PyInit__q_kangaroo)`.

**Why it happens:** One of the three naming locations was not updated consistently. Most commonly the `#[pymodule]` function name in lib.rs is forgotten, or the `module-name` in pyproject.toml still references the old name.

**How to avoid:** Change all three in the same commit, in this order:
1. `crates/qsym-python/Cargo.toml` `[lib] name = "_q_kangaroo"`
2. `crates/qsym-python/pyproject.toml` `module-name = "q_kangaroo._q_kangaroo"`
3. `crates/qsym-python/src/lib.rs` `fn _q_kangaroo(...)`

**Warning signs:** Build succeeds (`maturin develop` exits 0) but `import q_kangaroo` fails.

### Pitfall 2: Stale Compiled Artifacts (.pyd / __pycache__)

**What goes wrong:** After renaming the directory from `qsymbolic/` to `q_kangaroo/`, Python still finds and loads the old `_qsymbolic.pyd` from a stale location (e.g., site-packages or __pycache__).

**Why it happens:** `maturin develop` installs into the virtualenv's site-packages. The old `qsymbolic` package may still be installed there alongside the new `q_kangaroo`. Python finds `qsymbolic` first if test code hasn't been fully updated.

**How to avoid:**
1. Before running `maturin develop` with the new name, run `pip uninstall qsymbolic -y` to remove the old installation
2. Delete `__pycache__` directories: `find . -name __pycache__ -exec rm -rf {} +`
3. Delete old `.pyd` files from the source tree (they are build artifacts placed by maturin)
4. Run `maturin develop` fresh after cleanup

**Warning signs:** Tests pass but `import qsymbolic` ALSO still works (should fail after rename).

### Pitfall 3: Forgetting `pyproject.toml` `[project] name` vs `[tool.maturin] module-name`

**What goes wrong:** Developer updates `module-name` to `q_kangaroo._q_kangaroo` but leaves `[project] name = "qsymbolic"`. The package installs under the wrong name on PyPI.

**Why it happens:** These are two separate fields with different semantics. `[project] name` is the PyPI distribution name (user does `pip install <this>`). `module-name` is the Python import path for the native extension.

**How to avoid:** Update BOTH:
```toml
[project]
name = "q-kangaroo"        # PyPI name (hyphens)

[tool.maturin]
module-name = "q_kangaroo._q_kangaroo"  # Python import path (underscores)
```

### Pitfall 4: Git Not Tracking the Directory Rename

**What goes wrong:** Using plain `mv` instead of `git mv` for the directory rename means git sees it as a delete + add, losing file history.

**Why it happens:** On Windows/Cygwin, filesystem operations don't automatically go through git.

**How to avoid:** Use `git mv crates/qsym-python/python/qsymbolic crates/qsym-python/python/q_kangaroo` to rename the directory with git tracking.

### Pitfall 5: Cargo.lock Staleness

**What goes wrong:** After changing `[lib] name` in Cargo.toml, the Cargo.lock file may reference the old name, causing build confusion.

**Why it happens:** Cargo.lock caches package metadata. The `[lib] name` change affects the output artifact name but is tracked in Cargo.lock.

**How to avoid:** After editing Cargo.toml, run `cargo check -p qsym-python` (using the *package* name, not the lib name) to update Cargo.lock. Then verify with `cargo test`.

## Code Examples

### Cargo.toml Change (Verified from current source)

```toml
# BEFORE (current):
[lib]
name = "_qsymbolic"
crate-type = ["cdylib"]

# AFTER:
[lib]
name = "_q_kangaroo"
crate-type = ["cdylib"]
```

Source: `crates/qsym-python/Cargo.toml` line 8-9

### pyproject.toml Change (Verified from current source)

```toml
# BEFORE (current):
[project]
name = "qsymbolic"
version = "0.1.0"

[tool.maturin]
module-name = "qsymbolic._qsymbolic"

# AFTER:
[project]
name = "q-kangaroo"
version = "0.1.0"

[tool.maturin]
module-name = "q_kangaroo._q_kangaroo"
```

Source: `crates/qsym-python/pyproject.toml` lines 6, 14

### lib.rs Change (Verified from current source)

```rust
// BEFORE (current):
#[pymodule]
fn _qsymbolic(m: &Bound<'_, PyModule>) -> PyResult<()> {

// AFTER:
#[pymodule]
fn _q_kangaroo(m: &Bound<'_, PyModule>) -> PyResult<()> {
```

Source: `crates/qsym-python/src/lib.rs` line 22

### __init__.py Import Change (Verified from current source)

```python
# BEFORE (current):
from qsymbolic._qsymbolic import QSession, QExpr, QSeries, version
from qsymbolic._qsymbolic import (
    aqprod, qbin,
    # ... all other imports
)

# AFTER:
from q_kangaroo._q_kangaroo import QSession, QExpr, QSeries, version
from q_kangaroo._q_kangaroo import (
    aqprod, qbin,
    # ... all other imports
)
```

Source: `crates/qsym-python/python/qsymbolic/__init__.py` lines 28, 32

### test_integration.py Import Change (Verified from current source)

```python
# BEFORE (current, 9 occurrences in different test functions):
from qsymbolic import QSession, partition_gf, aqprod

# AFTER:
from q_kangaroo import QSession, partition_gf, aqprod
```

Source: `crates/qsym-python/tests/test_integration.py` lines 24, 51, 83, 109, 140, 160, 179, 217, 243

### Directory Rename Command

```bash
# Use git mv to preserve history:
git mv crates/qsym-python/python/qsymbolic crates/qsym-python/python/q_kangaroo
```

### Cleanup Commands

```bash
# Remove stale compiled artifacts:
rm -rf crates/qsym-python/python/q_kangaroo/__pycache__
rm -f crates/qsym-python/python/q_kangaroo/_qsymbolic.cp314-win_amd64.pyd

# Uninstall old package from venv:
pip uninstall qsymbolic -y 2>/dev/null || true

# Rebuild with new name:
export PATH="/c/mingw64-gcc/mingw64/bin:/c/cygwin64/bin:/c/Users/Owner/.cargo/bin:$PATH"
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
cd crates/qsym-python && maturin develop
```

### Verification Commands

```bash
# Verify Rust tests still pass (578 tests):
export PATH="/c/mingw64-gcc/mingw64/bin:/c/cygwin64/bin:/c/Users/Owner/.cargo/bin:$PATH"
cargo test --workspace

# Verify Python import works:
python -c "import q_kangaroo; print(q_kangaroo.__version__)"

# Verify old import fails (ensures clean rename):
python -c "import qsymbolic" 2>&1 | grep -q "ModuleNotFoundError" && echo "OLD NAME CORRECTLY REMOVED"

# Verify Python tests pass (9 tests):
cd crates/qsym-python && python -m pytest tests/test_integration.py -v

# Verify no remaining references to old name in source:
grep -r "qsymbolic" --include="*.py" --include="*.rs" --include="*.toml" crates/ && echo "FAIL: old name found" || echo "PASS: no old references"
```

## Execution Order (CRITICAL)

The rename must be done in this specific order to avoid intermediate broken states:

1. **Rename the Python directory** (`git mv qsymbolic q_kangaroo`) -- filesystem first
2. **Update __init__.py** -- fix the imports to use new module name
3. **Update Cargo.toml** -- change `[lib] name`
4. **Update pyproject.toml** -- change both `[project] name` and `module-name`
5. **Update lib.rs** -- change `#[pymodule]` function name
6. **Update test_integration.py** -- change all 9 import lines
7. **Clean stale artifacts** -- remove __pycache__, old .pyd, pip uninstall old
8. **Rebuild** -- `cargo test` then `maturin develop`
9. **Verify** -- run all Rust and Python tests

Steps 2-6 can be done in any order since they are all text edits. The key is that the directory rename (step 1) happens first and artifact cleanup (step 7) happens before rebuild (step 8).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `#[pymodule(name)]` | `#[pyo3(name = "...")]` on `#[pymodule]` fn | PyO3 0.17+ | Either works in 0.23; renaming the fn is simplest |
| Manual module-name in Cargo.toml metadata | `[tool.maturin] module-name` in pyproject.toml | maturin 0.13+ | pyproject.toml is the authoritative source now |

**Current PyO3 version (0.23):** Both approaches work for naming. The simplest is to just rename the function from `_qsymbolic` to `_q_kangaroo` -- no extra attribute needed. The `#[pyo3(name = "...")]` attribute is only needed if you want the Rust function name to differ from the Python module name.

## Open Questions

1. **Should PROJECT.md be updated as part of this phase?**
   - What we know: PROJECT.md contains 4 references to `qsymbolic` (code examples, architecture diagram). The success criterion says "source files, configs, or test code" -- PROJECT.md is a documentation file.
   - Recommendation: Update PROJECT.md as part of this phase for consistency. It is a source file in the repo and user-facing. LOW effort (4 simple text replacements).

2. **Should version be bumped from 0.1.0?**
   - What we know: Current version is 0.1.0 in both pyproject.toml and lib.rs `version()`. Phase 10 (PyPI packaging) will likely set version to 1.0.0.
   - Recommendation: Leave version as 0.1.0 in this phase. Version bump is Phase 10's concern.

3. **Should the docstring in __init__.py mention the new name?**
   - What we know: Current docstring says "Q-Symbolic: Symbolic computation for q-series"
   - Recommendation: Update to "q-Kangaroo: Symbolic computation for q-series" for consistency.

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection of all 5 affected files (read and verified current contents)
- [Maturin Project Layout documentation](https://www.maturin.rs/project_layout.html) -- module-name configuration
- [PyO3 #[pymodule] documentation](https://pyo3.rs/main/doc/pyo3/attr.pymodule) -- function naming
- [Cargo Workspaces documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html) -- members are directory paths, not package names

### Secondary (MEDIUM confidence)
- [Existing ARCHITECTURE.md research](file://.planning/research/ARCHITECTURE.md) -- rename migration flow already documented
- [Existing PITFALLS.md research](file://.planning/research/PITFALLS.md) -- triple-match pitfall previously identified

### Tertiary (LOW confidence)
- None. All findings verified against actual codebase.

## Metadata

**Confidence breakdown:**
- File change list: HIGH -- exhaustive grep + direct file reads confirm exactly 5 source files + 1 directory
- Triple-match rule: HIGH -- verified against maturin docs and existing working configuration
- Execution order: HIGH -- based on direct analysis of dependency chain
- Scope (no Rust crate rename): HIGH -- requirements explicitly scope to `qsymbolic` only

**Research date:** 2026-02-14
**Valid until:** Indefinite (naming conventions are stable; no external dependencies changing)
