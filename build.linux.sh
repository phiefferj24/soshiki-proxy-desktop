aarch64-unknown-linux-gnu i686-unknown-linux-gnu x86_64-unknown-linux-gnu
rm -r ./out/linux/*
mkdir -p out/linux
for target in aarch64-apple-darwin x86_64-apple-darwin; do
    rustup target add $target
    cargo build --release --target $target
    mkdir -p ./out/linux/$target
    cp ./target/$target/release/soshiki-proxy-desktop* ./out/linux/$target/
    rm ./out/linux/$target/soshiki-proxy-desktop.d
done
exit 0