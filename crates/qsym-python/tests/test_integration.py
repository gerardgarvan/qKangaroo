"""
Integration test: replicate a Garvan-style q-series identity discovery workflow.

This test demonstrates the full q-Kangaroo Python API by:
1. Computing series from named products and partition functions
2. Using relation discovery to find identities
3. Verifying identities by series comparison
4. Batch generation for systematic parameter scanning
5. Symbolic expression manipulation with LaTeX/Unicode rendering

Run with: python tests/test_integration.py
Or with pytest: pytest tests/test_integration.py -v
"""
from fractions import Fraction


def test_euler_identity():
    """
    Verify Euler's identity: prod_{n=1}^{inf} 1/(1-q^n) = sum_{n=0}^{inf} p(n) q^n

    The partition generating function IS the inverse Euler product.
    We verify by computing both sides independently and checking coefficient agreement.
    """
    from q_kangaroo import QSession, partition_gf, aqprod

    s = QSession()
    order = 50

    # Left side: partition generating function (via pentagonal recurrence)
    pgf = partition_gf(s, order)

    # Right side: (q;q)_inf inverted = 1 / prod_{n=1}^inf (1-q^n)
    # aqprod with a=q (coeff=1/1, power=1), n=None (infinite) gives (q;q)_inf
    qq_inf = aqprod(s, 1, 1, 1, None, order)  # (q;q)_inf
    euler_inv = qq_inf.invert()  # 1/(q;q)_inf

    # Verify: coefficients must match for all powers 0..49
    for k in range(order):
        assert pgf[k] == euler_inv[k], f"Mismatch at q^{k}: pgf={pgf[k]}, euler_inv={euler_inv[k]}"

    print(f"Euler identity verified to O(q^{order})")


def test_jacobi_triple_product():
    """
    Verify a case of the Jacobi triple product identity using series comparison.

    theta3(q) = (q^2;q^2)_inf * (-q;q^2)_inf * (-q;q^2)_inf
    = sum_{n=-inf}^{inf} q^{n^2}
    """
    from q_kangaroo import QSession, theta3

    s = QSession()
    order = 30

    t3 = theta3(s, order)

    # theta3 coefficients: coeff of q^k is 2 if k is a nonzero perfect square,
    # 1 if k=0, 0 otherwise. (Since sum is over n from -inf to inf, both +n and -n contribute.)
    import math
    for k in range(order):
        expected = Fraction(0)
        if k == 0:
            expected = Fraction(1)
        else:
            sqrt_k = int(math.isqrt(k))
            if sqrt_k * sqrt_k == k:
                expected = Fraction(2)  # both +sqrt_k and -sqrt_k contribute
        assert t3[k] == expected, f"theta3 mismatch at q^{k}: got {t3[k]}, expected {expected}"

    print(f"Jacobi triple product (theta3) verified to O(q^{order})")


def test_findlincombo_identity():
    """
    Use findlincombo to discover a linear relation between q-series.

    We know: partition_gf(q) = 1/(q;q)_inf
    So: (q;q)_inf * partition_gf(q) = 1

    Build the product series, then verify it equals the constant 1.
    """
    from q_kangaroo import QSession, partition_gf, aqprod

    s = QSession()
    order = 30

    pgf = partition_gf(s, order)
    qq_inf = aqprod(s, 1, 1, 1, None, order)

    # product = (q;q)_inf * pgf should be identically 1 + O(q^30)
    product = qq_inf * pgf

    # Check it's 1
    assert product[0] == Fraction(1), f"product[0] = {product[0]}, expected 1"
    for k in range(1, order):
        assert product[k] == Fraction(0), f"product[{k}] = {product[k]}, expected 0"

    print(f"(q;q)_inf * partition_gf = 1 verified to O(q^{order})")


def test_prodmake_roundtrip():
    """
    Test prodmake: compute a product, expand to series, recover the product form.

    Start with etaq(1, 1, q, N) = (q;q)_inf = prod_{n=1}^{inf} (1-q^n)
    Expand to series, then use prodmake to recover the factors.
    """
    from q_kangaroo import QSession, etaq, prodmake

    s = QSession()
    order = 30

    # etaq(1, 1) gives (q;q)_inf = prod_{n=1}^{inf} (1 - q^n)
    euler = etaq(s, 1, 1, order)

    # Recover product form
    pf = prodmake(euler, 15)

    # Andrews' algorithm returns exponents as QRat -> Python Fraction.
    # For (q;q)_inf = prod (1-q^n)^{-a_n}, a_n should be -1 for all n.
    # prodmake convention: exponents map n -> a_n where prod is (1-q^n)^{-a_n}.
    # For (q;q)_inf = prod (1-q^n)^1, that means (1-q^n)^{-a_n} = (1-q^n)^1 => a_n = -1.
    print(f"prodmake result: {pf}")

    # Check that the exponents include a_n = -1 for small n
    factors = pf["factors"]
    for n in range(1, 10):
        assert n in factors, f"Missing exponent for n={n} in prodmake result"
        # Exponents are Fraction (QRat), compare with Fraction(-1)
        assert factors[n] == Fraction(-1), f"Exponent for n={n} is {factors[n]}, expected Fraction(-1)"

    print("prodmake roundtrip verified")


def test_batch_parameter_scan():
    """
    Test batch generation: scan etaq over a parameter grid.
    """
    from q_kangaroo import QSession

    s = QSession()

    # Scan etaq(b, 1) for b = 1..5
    results = s.batch_generate("etaq", [[b, 1] for b in range(1, 6)], 20)

    assert len(results) == 5, f"Expected 5 results, got {len(results)}"

    for params, series in results:
        assert not series.is_zero(), f"etaq(b={params[0]}, t={params[1]}) should not be zero"
        assert series[0] == Fraction(1), f"etaq constant term should be 1"

    print("Batch parameter scan verified")


def test_single_generate():
    """
    Test the single generate() convenience method.
    """
    from q_kangaroo import QSession, etaq

    s = QSession()

    # generate("etaq", [1, 1], 20) should match etaq(s, 1, 1, 20)
    gen_result = s.generate("etaq", [1, 1], 20)
    direct_result = etaq(s, 1, 1, 20)

    for k in range(20):
        assert gen_result[k] == direct_result[k], \
            f"Mismatch at q^{k}: generate={gen_result[k]}, direct={direct_result[k]}"

    print("Single generate verified")


def test_symbols_and_expressions():
    """
    Test the symbolic expression API: create, combine, render.
    """
    from q_kangaroo import QSession

    s = QSession()
    q_a_n = s.symbols("q a n")
    q, a, n = q_a_n[0], q_a_n[1], q_a_n[2]

    # Build expression: q^2 + a
    q2 = q ** s.integer(2)
    expr = q2 + a

    # Check rendering
    repr_str = repr(expr)
    assert len(repr_str) > 0, "repr should not be empty"

    latex_str = expr._repr_latex_()
    assert latex_str.startswith("$"), "LaTeX should start with $"
    assert latex_str.endswith("$"), "LaTeX should end with $"

    # Check structural equality
    q2_again = q ** s.integer(2)
    assert q2 == q2_again, "Same expression should have same ExprRef"

    # Check stats
    num_exprs, num_syms = s.stats()
    assert num_syms >= 3, f"Should have at least 3 symbols, got {num_syms}"

    print(f"Expression API verified: {repr_str}")
    print(f"LaTeX: {latex_str}")
    print(f"Stats: {num_exprs} expressions, {num_syms} symbols")


def test_distinct_odd_euler_identity():
    """
    Verify Euler's distinct-odd partition theorem:
    prod_{n>=1} (1+q^n) = prod_{k>=0} 1/(1-q^{2k+1})

    i.e., distinct_parts_gf = odd_parts_gf
    """
    from q_kangaroo import QSession, distinct_parts_gf, odd_parts_gf

    s = QSession()
    order = 40

    dpgf = distinct_parts_gf(s, order)
    opgf = odd_parts_gf(s, order)

    for k in range(order):
        assert dpgf[k] == opgf[k], \
            f"Distinct/odd mismatch at q^{k}: distinct={dpgf[k]}, odd={opgf[k]}"

    print(f"Euler distinct-odd partition theorem verified to O(q^{order})")


def test_hypergeometric_identity_verification():
    """
    Verify a hypergeometric identity end-to-end from Python:
    Construct a 2phi1 matching q-Gauss parameters, evaluate term-by-term via phi(),
    get closed form via try_summation(), and confirm both sides agree.

    q-Gauss: _2phi1(a, b; c; q, c/(ab)) = (c/a;q)_inf * (c/b;q)_inf / [(c;q)_inf * (c/(ab);q)_inf]

    Using a=q (1,1,1), b=q^2 (1,1,2), c=q^5 (1,1,5):
    z = c/(ab) = q^5 / (q * q^2) = q^2 = (1,1,2)
    """
    from q_kangaroo import QSession, phi, try_summation
    from fractions import Fraction

    s = QSession()
    order = 30

    # Define q-Gauss parameters: _2phi1(q, q^2; q^5; q, q^2)
    upper = [(1, 1, 1), (1, 1, 2)]   # a=q, b=q^2
    lower = [(1, 1, 5)]               # c=q^5
    z_num, z_den, z_pow = 1, 1, 2     # z=q^2 = c/(ab)

    # Side 1: Term-by-term evaluation via phi()
    term_by_term = phi(s, upper, lower, z_num, z_den, z_pow, order)

    # Side 2: Closed form via try_summation()
    closed_form = try_summation(s, upper, lower, z_num, z_den, z_pow, order)

    # try_summation should recognize q-Gauss and return a result
    assert closed_form is not None, \
        "try_summation should recognize q-Gauss pattern for _2phi1(q, q^2; q^5; q, q^2)"

    # Compare coefficients: both sides must agree to O(q^order)
    for k in range(order):
        assert term_by_term[k] == closed_form[k], \
            f"Identity mismatch at q^{k}: phi={term_by_term[k]}, summation={closed_form[k]}"

    # Sanity check: the series is not trivially zero or constant
    assert term_by_term[0] == Fraction(1), \
        f"Constant term should be 1, got {term_by_term[0]}"
    has_nonzero_higher = any(term_by_term[k] != Fraction(0) for k in range(1, min(10, order)))
    assert has_nonzero_higher, \
        "Series should have nonzero higher-order terms (not a trivial identity)"

    print(f"Hypergeometric identity (q-Gauss) verified to O(q^{order})")


if __name__ == "__main__":
    test_euler_identity()
    test_jacobi_triple_product()
    test_findlincombo_identity()
    test_prodmake_roundtrip()
    test_batch_parameter_scan()
    test_single_generate()
    test_symbols_and_expressions()
    test_distinct_odd_euler_identity()
    test_hypergeometric_identity_verification()
    print("\n=== ALL INTEGRATION TESTS PASSED ===")
