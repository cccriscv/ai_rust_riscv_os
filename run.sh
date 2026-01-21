set -x
cd user_app
./build.sh
cd ../mkfs
./makefs.sh
cd ../eos1
cargo build
./run.sh
