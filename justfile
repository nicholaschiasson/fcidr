build *ARGS:
  cargo build {{ARGS}}

check *ARGS:
  cargo check {{ARGS}}

clean *ARGS:
  cargo clean {{ARGS}}

format *ARGS:
  cargo fmt {{ARGS}}

lint *ARGS:
  cargo clippy {{ARGS}}

run *ARGS:
  cargo run {{ARGS}}

test *ARGS:
  cargo test {{ARGS}}

watch *ARGS:
  cargo watch {{ARGS}}
