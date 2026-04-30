-- expModInteger 0 (-1) 1 should return 0 (Haskell special-cases m==1)
(program 1.1.0
    [(builtin expModInteger) (con integer 0) (con integer -1) (con integer 1)]
)