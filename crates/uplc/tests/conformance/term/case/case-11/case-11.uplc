-- case on True with only 1 branch — should fail (True needs branch index 1)
(program 1.1.0
    (case (con bool True) (con integer 42))
)