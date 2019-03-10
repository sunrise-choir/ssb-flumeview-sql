mkdir -p ./build/libsodium/lib
mkdir -p ./build/Release

cd native

cargo make build 
