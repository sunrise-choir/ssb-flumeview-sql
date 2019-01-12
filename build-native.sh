
mkdir -p ./build/libsodium/lib
mkdir -p ./build/Release

export SODIUM_STATIC=1 
export SODIUM_INSTALL_DIR=$PWD/build/libsodium
export SODIUM_LIB_DIR=$SODIUM_INSTALL_DIR/lib

cd native

# cargo make no-cross
#cargo make no-cross-debug
cargo make cross \
  --env DOCKER_CROSS_IMAGE_NAME="linux-x64"\
  --env DOCKER_CROSS_TRIPLE="x86_64-unknown-linux-gnu"\
  --env CROSS_TRIPLE="x86_64-unknown-linux-gnu"

# cargo make cross \
#   --env DOCKER_CROSS_IMAGE_NAME="linux-arm64"\
#   --env DOCKER_CROSS_TRIPLE="aarch64-unknown-linux-gnu"\
#   --env CROSS_TRIPLE="aarch64-unknown-linux-gnu" && \

# This fails because legacy-msg-data doesn't support 32 bit processors at the moment 
# cargo make cross \
#   --env DOCKER_CROSS_IMAGE_NAME="linux-armv7"\
#   --env DOCKER_CROSS_TRIPLE="armv7-unknown-linux-gnueabi"\
#   --env CROSS_TRIPLE="armv7-unknown-linux-gnueabihf"

# This fails because cmake can't do the final link to napi symbols. Not sure how to resolve.
# cargo make cross \
#   --env DOCKER_CROSS_IMAGE_NAME="windows-x64"\
#   --env DOCKER_CROSS_TRIPLE="x86_64-pc-windows-gnu"\
#   --env CROSS_TRIPLE="x86_64-pc-windows-gnu" && \
