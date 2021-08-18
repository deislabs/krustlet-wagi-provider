export RUST_LOG := "wagi_provider=debug,main=debug,kubelet=debug"
export CONFIG_DIR := env_var_or_default('CONFIG_DIR', '$HOME/.krustlet/config')

build +FLAGS='':
    cargo build {{FLAGS}}

run +FLAGS='': bootstrap
    KUBECONFIG=$(eval echo $CONFIG_DIR)/kubeconfig-wagi cargo run --bin krustlet-wagi {{FLAGS}} -- --node-name krustlet-wagi --port 3001 --bootstrap-file $(eval echo $CONFIG_DIR)/bootstrap.conf --cert-file $(eval echo $CONFIG_DIR)/krustlet-wagi.crt --private-key-file $(eval echo $CONFIG_DIR)/krustlet-wagi.key

bootstrap:
    @# This is to get around an issue with the default function returning a string that gets escaped
    @mkdir -p $(eval echo $CONFIG_DIR)
    @test -f  $(eval echo $CONFIG_DIR)/bootstrap.conf || CONFIG_DIR=$(eval echo $CONFIG_DIR) ./scripts/bootstrap.sh
    @chmod 600 $(eval echo $CONFIG_DIR)/*