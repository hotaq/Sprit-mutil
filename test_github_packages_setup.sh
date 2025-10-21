#!/bin/bash

# Test script to validate GitHub Packages setup locally
# This script validates the configuration before pushing to CI

set -e

echo "🧪 Testing GitHub Packages Setup..."
echo "==================================="

# Check if Cargo.toml has the required fields
echo "1️⃣  Checking Cargo.toml configuration..."
if grep -q "publish = \[\"github\"\]" Cargo.toml; then
    echo "✅ publish = [\"github\"] found in Cargo.toml"
else
    echo "❌ publish = [\"github\"] not found in Cargo.toml"
    exit 1
fi

# Check if workflow file exists
echo ""
echo "2️⃣  Checking GitHub Actions workflow..."
if [ -f ".github/workflows/publish-github-packages.yml" ]; then
    echo "✅ GitHub Packages workflow exists"
else
    echo "❌ GitHub Packages workflow missing"
    exit 1
fi

# Check if documentation exists
echo ""
echo "3️⃣  Checking documentation..."
if [ -f "docs/GITHUB_PACKAGES.md" ]; then
    echo "✅ GitHub Packages documentation exists"
else
    echo "❌ GitHub Packages documentation missing"
    exit 1
fi

# Validate Cargo.toml format
echo ""
echo "4️⃣  Validating Cargo.toml format..."
if cargo check --quiet; then
    echo "✅ Cargo.toml is valid"
else
    echo "❌ Cargo.toml has syntax errors"
    exit 1
fi

# Test local build
echo ""
echo "5️⃣  Testing local build..."
if cargo build --quiet; then
    echo "✅ Local build successful"
else
    echo "❌ Local build failed"
    exit 1
fi

# Check if binary works
echo ""
echo "6️⃣  Testing binary functionality..."
if ./target/debug/sprite --version > /dev/null 2>&1; then
    VERSION=$(./target/debug/sprite --version)
    echo "✅ Binary works: $VERSION"
else
    echo "❌ Binary doesn't work"
    exit 1
fi

echo ""
echo "🎉 All GitHub Packages setup tests passed!"
echo ""
echo "📦 Next steps:"
echo "   1. Push to main branch to trigger GitHub Packages publishing"
echo "   2. Check GitHub Actions tab for workflow status"
echo "   3. Verify package appears in GitHub Packages registry"
echo "   4. Test installation: cargo install --registry github --git <repo> sprite"
echo ""
echo "📖 For detailed instructions, see: docs/GITHUB_PACKAGES.md"