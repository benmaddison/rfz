#!/usr/bin/env bash

set -e

_usage() {
    cat <<EOF
usage:
    ./test.sh [lint|cover|bench|all]
EOF
}

_test() {
    cargo test
}

_lint() {
    cargo clippy -- -D warnings
    cargo fmt -- --check
}

_coverage() {
    cargo="cargo +nightly"
    output=".coverage"

    export RUSTFLAGS="-Zinstrument-coverage"

    ${cargo} clean
    ${cargo} test --lib 
    rustup run nightly grcov . --binary-path target/debug/ \
                               --source-dir . \
                               --branch \
                               --ignore-not-existing \
                               --ignore "/*" \
                               --excl-line 'panic!' \
                               --output-type "html" \
                               --output-path "${output}"
    echo "Opening report in browser"
    ${BROWSER:-firefox} "${output}/src/index.html"
}

_bench() {
    cargo build --release
    hyperfine 'target/release/rfz index'
}

cmd="$1"
case "${cmd}" in
    "")
        _test;;
    lint)
        _lint;;
    cover)
        _coverage;;
    bench)
        _bench;;
    all)
        _test
        _lint
        _bench
        _coverage
        ;;
    *)
        _usage
        exit 1
        ;;
esac
