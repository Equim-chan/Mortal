set -ex
cargo build -p libriichi --lib
cp target/debug/libriichi.so ./mortal/