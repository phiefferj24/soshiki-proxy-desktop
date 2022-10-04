rm -r ./out/mac/*
mkdir -p out/mac
for target in aarch64-apple-darwin x86_64-apple-darwin; do
    cargo build --release --target $target
    mkdir -p ./out/mac/$target
    cp ./target/$target/release/soshiki-proxy-desktop* ./out/mac/$target/
    rm ./out/mac/$target/soshiki-proxy-desktop.d
done
exit 0