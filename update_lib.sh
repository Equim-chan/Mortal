set -ex
rustup run nightly cargo build -p libriichi --lib --release
cp target/release/libriichi.so ./mortal/