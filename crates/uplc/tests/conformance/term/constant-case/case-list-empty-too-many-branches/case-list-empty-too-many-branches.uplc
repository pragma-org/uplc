-- case on empty list with too many branches (4) should fail;
-- lists support exactly 1 (cons-only) or 2 (cons + nil) branches
(program 1.1.0
    (case (con (list integer) [])
        (con integer 1)
        (con integer 2)
        (con integer 3)
        (con integer 4))
)
