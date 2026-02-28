#!/bin/bash

# OpenCode Web Startup Script for GitHub Codespaces

set -e

# Build the access URL
if [ -n "$CODESPACE_NAME" ] && [ -n "$GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN" ]; then
    ACCESS_URL="https://${CODESPACE_NAME}-3000.${GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN}"
else
    # Fallback for local development or missing env vars
    ACCESS_URL="http://localhost:3000"
fi

# Display startup message
echo ""
echo "============================================"
echo "🚀 OpenCode Web is starting..."
echo ""
echo "📱 Access URL:"
echo ""
echo "$ACCESS_URL"
echo ""
echo "============================================"
echo ""

# Set working directory (Codespaces-only)
# Determine repo root from this script location to avoid depending on environment variables.
WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Create workspace directory if it doesn't exist
mkdir -p "$WORKSPACE_DIR"

# Start OpenCode Web
cd "$WORKSPACE_DIR"
exec npx -y opencode-ai@latest web --mdns --port 3000
