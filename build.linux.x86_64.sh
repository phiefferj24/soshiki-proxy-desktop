for target in x86_64-unknown-linux-gnu i686-unknown-linux-gnu; do
    rustup target add $target
    cargo build --release --target $target
    mkdir -p ./out/linux/$target
    rm -r ./out/linux/$target/*
    cp ./target/$target/release/soshiki-proxy-desktop* ./out/linux/$target/
    rm ./out/linux/$target/soshiki-proxy-desktop.d
done
exit 0
