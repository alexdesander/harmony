# Clean or make build dir
[ -d "artifacts" ] && rm -rf "artifacts"/* || mkdir -p "artifacts"

# Build backend
echo "Building server..."
cd ./server
cargo build --release
cd ..
mkdir artifacts/backend
mv ./target/release/server ./artifacts/backend/server