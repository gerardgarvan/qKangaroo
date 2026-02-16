Algorithmic Summation & Identity Proving
=========================================

The q-Gosper algorithm performs indefinite q-hypergeometric summation.
The q-Zeilberger algorithm extends this via creative telescoping to find
recurrences for definite sums :math:`S(n) = \sum_k F(n,k)`. The q-Petkovsek
algorithm solves the resulting recurrences for closed-form solutions.
The WZ (Wilf-Zeilberger) certificate provides a proof certificate that
can be independently verified. The Chen-Hou-Mu method proves nonterminating
identities by parameter specialization, and the transformation chain search
discovers sequences of Heine/Sears/Watson transformations connecting two
hypergeometric series.

.. autofunction:: q_kangaroo.q_gosper

.. autofunction:: q_kangaroo.q_zeilberger

.. autofunction:: q_kangaroo.verify_wz

.. autofunction:: q_kangaroo.q_petkovsek

.. autofunction:: q_kangaroo.prove_nonterminating

.. autofunction:: q_kangaroo.find_transformation_chain
