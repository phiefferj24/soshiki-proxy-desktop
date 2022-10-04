rm -r ./out/win/*
mkdir -p out/win
foreach ($target in "aarch64-pc-windows-msvc","x86_64-pc-windows-gnu","i686-pc-windows-gnu") {
    cargo build --release --target $target
    mkdir -p ./out/win/$target
    cp ./target/$target/release/soshiki-proxy-desktop* ./out/win/$target/
    rm ./out/win/$target/soshiki-proxy-desktop.d
}
exit 0