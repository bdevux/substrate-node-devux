cargo run --release \-- \
  purge-chain \
  --base-path /tmp/alice \
  --chain=local

cargo run --release \-- \
  purge-chain \
  --base-path /tmp/bob \
  --chain=local
