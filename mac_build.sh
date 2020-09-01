#! /bin/sh

cargo build --release --lib
cbindgen -l C src/mac_lib.rs > ./mac/Mixolumia\ Mod\ Installer/include/mod_installer.h

cp ./target/release/libmod_installer.a ./mac/Mixolumia\ Mod\ Installer/libs