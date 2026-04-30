-- consByteString 255 #00 under V2 semantics (should bounds-check, 255 is valid)
(program 1.1.0
    [(builtin consByteString) (con integer 255) (con bytestring #00)]
)