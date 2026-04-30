-- expModInteger 5 (-1) 1 should return 0 (anything mod 1 is 0)
(program 1.1.0
    [(builtin expModInteger) (con integer 5) (con integer -1) (con integer 1)]
)