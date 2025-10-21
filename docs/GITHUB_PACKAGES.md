# GitHub Packages Installation

Sprite is available on GitHub Packages, which provides secure private and public package hosting for Rust crates.

## Installation Methods

### Method 1: Using cargo install with GitHub Packages

```bash
# Install directly from GitHub Packages
cargo install --registry github --git https://github.com/hotaq/Sprit-mutil.git sprite

# Or install a specific version
cargo install --registry github --git https://github.com/hotaq/Sprit-mutil.git --tag v0.2.3 sprite
```

### Method 2: Adding to Cargo.toml

Add this to your `Cargo.toml`:

```toml
[dependencies]
sprite = { version = "0.2.3", registry = "github" }
```

You'll also need to configure your `.cargo/config.toml` to use GitHub Packages:

```toml
# In ~/.cargo/config.toml or .cargo/config.toml
[registries.github]
index = "sparse+https://github.com/hotaq/Sprit-mutil.git/registry/index/"
```

### Method 3: Using GitHub Container Registry

For environments that prefer container-based deployment:

```bash
# Pull the container image (if available)
docker pull ghcr.io/hotaq/sprit-mutil:latest
```

## Authentication

GitHub Packages requires authentication for both public and private packages.

### Setting up Authentication

1. **Create a GitHub Personal Access Token**:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Create a token with `read:packages` scope for downloading
   - Create a token with `write:packages` scope for publishing

2. **Configure Cargo**:
   ```bash
   # Login to GitHub Packages
   cargo login --registry github <YOUR_GITHUB_TOKEN>
   ```

3. **Environment Variable** (Alternative):
   ```bash
   export CARGO_REGISTRY_TOKEN=your_github_token
   ```

## Publishing

This project is configured to automatically publish to GitHub Packages when:

- Code is pushed to the `main` or `001-multi-agent-toolkit` branch
- A new release is published
- The workflow is manually triggered

### Manual Publishing

To publish manually:

```bash
# Ensure you're logged in
cargo login --registry github <YOUR_TOKEN>

# Publish to GitHub Packages
cargo publish --registry github
```

## Package Information

- **Package Name**: `sprite`
- **Registry**: `github`
- **Repository**: https://github.com/hotaq/Sprit-mutil
- **Package URL**: https://github.com/hotaq/Sprit-mutil/pkgs/container/sprite

## Version Management

GitHub Packages follows semantic versioning. Available versions:

- `v0.2.3` - Latest stable release (with test fixes and agent activation)
- `v0.2.2` - Previous stable release
- Earlier versions may be available but are not recommended

## Troubleshooting

### Common Issues

1. **Authentication Errors**:
   ```
   error: failed to publish to registry: authentication failed
   ```
   **Solution**: Ensure your GitHub token has the correct permissions (`write:packages`).

2. **Registry Not Found**:
   ```
   error: registry 'github' is not configured
   ```
   **Solution**: Configure the GitHub registry in your `.cargo/config.toml`.

3. **Network Issues**:
   ```
   error: failed to get package information
   ```
   **Solution**: Check your internet connection and GitHub status.

### Debug Mode

Enable verbose output for debugging:

```bash
cargo install --registry github --verbose sprite
```

## Integration with CI/CD

The GitHub Actions workflow (`.github/workflows/publish-github-packages.yml`) handles automatic publishing. It:

1. Runs comprehensive tests
2. Builds the package for multiple targets
3. Publishes to GitHub Packages
4. Verifies package availability

## Security Considerations

- Use repository secrets for tokens
- Limit token scopes to minimum required permissions
- Consider using environment-specific tokens
- Regularly rotate authentication tokens

## Support

For issues with GitHub Packages installation:

1. Check the [GitHub Packages documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-rust-registry-for-github-packages)
2. [Open an issue](https://github.com/hotaq/Sprit-mutil/issues) in this repository
3. Check existing [discussions](https://github.com/hotaq/Sprit-mutil/discussions)