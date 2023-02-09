#!/bin/bash
set -x
cargo build --release
sudo systemctl stop ya_web3_proxy
sudo cp target/release/ya_web3_proxy /usr/bin/ya_web3_proxy
sudo systemctl start ya_web3_proxy
# check if new version is properly installed
sleep 1
curl http://127.0.0.1:8546
echo Finished