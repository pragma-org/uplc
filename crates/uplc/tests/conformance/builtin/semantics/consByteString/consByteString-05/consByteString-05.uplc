-- consByteString 256 #00 under V2 semantics (should fail: 256 > 255)
(program 1.1.0
    [(builtin consByteString) (con integer 256) (con bytestring #00)]
)