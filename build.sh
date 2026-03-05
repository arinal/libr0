#!/bin/bash
# Build script for libr0 - creates custom landing page as index

# Build the book
mdbook build

# Save the generated Introduction page
mv book/index.html book/introduction.html

# Use our custom landing page as the index
cp theme/landing.html book/index.html

# Replace the default favicon with our custom libr0 logo
cp theme/libr0-logo.svg book/favicon.svg

echo "✓ Book built successfully with custom landing page"
echo "  - Landing page: book/index.html"
echo "  - Introduction: book/introduction.html"
echo "  - Favicon: book/favicon.svg"

# If 'serve' argument provided, start a local server
if [ "$1" = "serve" ]; then
    echo ""
    echo "Starting local server at http://localhost:8000"
    echo "Press Ctrl+C to stop"
    cd book && python3 -m http.server 8000
fi