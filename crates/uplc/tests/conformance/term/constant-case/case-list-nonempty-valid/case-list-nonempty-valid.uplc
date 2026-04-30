-- case on non-empty list with 2 branches: takes the cons branch (index 0),
-- binding head and tail as arguments
(program 1.1.0
    (case (con (list integer) [42, 99])
        (lam h (lam t h))
        (con integer 0))
)
