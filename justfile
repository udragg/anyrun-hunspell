install:
    cargo build --release
    cp ./target/release/libhunspell.so ~/.config/anyrun/plugins/

install_default_config *args="":
    cargo run --release --example default_config -- {{args}}
