-- consByteString (-1) #00 under V2 semantics (should fail: -1 < 0)
(program 1.1.0
    [(builtin consByteString) (con integer -1) (con bytestring #00)]
)