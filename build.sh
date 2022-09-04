cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu --release
cargo build --release

upx --best --lzma "target/release/rgrep"
upx --best --lzma "target/x86_64-unknown-linux-gnu/release/rgrep"