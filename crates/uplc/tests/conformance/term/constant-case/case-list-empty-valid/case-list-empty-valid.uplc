-- case on empty list with exactly 2 branches: takes the nil branch (index 1)
(program 1.1.0
    (case (con (list integer) [])
        (con integer 1)
        (con integer 2))
)
