test:
    cargo nextest run
    cargo bench

download-plutus-tests:
    set -euo pipefail
    rm -rf crates/uplc/tests/conformance
    curl -L -s https://github.com/IntersectMBO/plutus/archive/master.tar.gz | tar xz -C /tmp
    mkdir -p crates/uplc/tests/conformance
    mv /tmp/plutus-master/plutus-conformance/test-cases/uplc/evaluation/* crates/uplc/tests/conformance/
    rm -rf /tmp/plutus-master
    mv crates/uplc/tests/conformance/builtin/constant/unit/conUnit.uplc crates/uplc/tests/conformance/builtin/constant/unit/unit.uplc
    rm crates/uplc/tests/conformance/builtin/constant/unit/conUnit.uplc.expected
    echo "Download complete. Test cases are now in crates/uplc/tests/conformance/"
