# Clean or make build dir
[ -d "artifacts" ] && rm -rf "artifacts"/* || mkdir -p "artifacts"

# Build backend
echo "Building server..."
cd ./server
cargo build --release
cd ..
mkdir artifacts/backend
mv ./target/release/server ./artifacts/backend/server

# Build client
echo "Building client.."
cd ./client
trunk build --release
cd ..
mkdir artifacts/client
mv ./client/dist ./artifacts/client/dist
cd ..