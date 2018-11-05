mkdir -p ./build/libsodium/lib

export SODIUM_STATIC=1 
export SODIUM_INSTALL_DIR=$PWD/build/libsodium
export SODIUM_LIB_DIR=$SODIUM_INSTALL_DIR/lib

cd native

cargo build --release 
