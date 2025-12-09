# Format code (required before commits)
echo "Formatting code..."
make format

# Check formatting
echo "Checking code formatting..."
make check-format

# Run linter (clippy)
echo "Running linter..."
make lint

# Fix trailing whitespaces (mandatory before commits)
echo "Fixing trailing whitespaces..."
make fix-trailing-whitespace

# Check for trailing whitespaces
echo "Checking for trailing whitespaces..."
make check-trailing-whitespace
