cargo run --release \-- \
  --base-path /tmp/alice \
  --chain=local \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --alice \
  --validator
  
# cargo run --release \-- \
#   purge-chain \
#   --base-path /tmp/alice \
#   --chain=local
