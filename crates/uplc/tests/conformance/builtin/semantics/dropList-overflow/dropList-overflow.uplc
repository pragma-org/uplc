-- dropList with a huge integer should exceed budget due to saturating cost,
-- not succeed with a wrapped-negative cost
(program 1.1.0
    [[(force (builtin dropList)) (con integer 7375927274556299409)]
     (con (list data) [(I 1), (I 2), (I 3)])]
)
