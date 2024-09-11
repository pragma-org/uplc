test:
    cargo nextest run
    cargo bench

download-plutus-tests:
    set -euo pipefail
    rm -rf tests/conformance
    echo "Downloading Plutus repository..."
    curl -L https://github.com/IntersectMBO/plutus/archive/master.tar.gz | tar xz -C /tmp
    echo "Moving specific test cases..."
    mkdir -p tests/conformance
    mv /tmp/plutus-master/plutus-conformance/test-cases/uplc/evaluation/* tests/conformance/
    echo "Cleaning up..."
    rm -rf /tmp/plutus-master
    echo "Download complete. Test cases are now in tests/conformance/"
