#!/bin/bash
set -e

echo "🚀 Building binaries with cargo zigbuild..."
# Build for x86_64 (Intel Linux)
echo "   - Building x86_64-unknown-linux-musl..."
cargo zigbuild --release --target x86_64-unknown-linux-musl -p port_sentinel_master
cargo zigbuild --release --target x86_64-unknown-linux-musl -p port_sentinel_agent

# Build for aarch64 (ARM Linux)
echo "   - Building aarch64-unknown-linux-musl..."
cargo zigbuild --release --target aarch64-unknown-linux-musl -p port_sentinel_master
cargo zigbuild --release --target aarch64-unknown-linux-musl -p port_sentinel_agent

echo "📦 Updating assets and installer in dist folders..."
# Ensure directories exist
mkdir -p dist/assets dist_arm/assets

# Copy Binaries
cp target/x86_64-unknown-linux-musl/release/port_sentinel_master dist/
cp target/x86_64-unknown-linux-musl/release/port_sentinel_agent dist/

cp target/aarch64-unknown-linux-musl/release/port_sentinel_master dist_arm/
cp target/aarch64-unknown-linux-musl/release/port_sentinel_agent dist_arm/

# Copy assets
cp -r master/assets/* dist/assets/
cp -r master/assets/* dist_arm/assets/

# Copy installer
cp install.sh dist/
cp install.sh dist_arm/

echo "🏗️  Creating x86_64 bundle..."
tar -czvf port_sentinel_bundle_x86_64.tar.gz -C dist .

echo "🏗️  Creating aarch64 bundle..."
tar -czvf port_sentinel_bundle_aarch64.tar.gz -C dist_arm .

echo "✅ Bundling Complete"
