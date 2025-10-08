#!/bin/bash

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./bump-version.sh 0.1.0"
  exit 1
fi

# Update package.json
echo "Updating package.json..."
bun version $VERSION --no-git-tag-version

# Update Cargo.toml
echo "Updating Cargo.toml..."
sed -i '' "s/^version = .*/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Update tauri.conf.json
echo "Updating tauri.conf.json..."
sed -i '' "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

echo ""
echo "✅ Version bumped to $VERSION"
echo ""
echo "Next steps:"
echo "  git add ."
echo "  git commit -m 'chore: bump version to v$VERSION'"
echo "  git tag v$VERSION"
echo "  git push && git push --tags"
