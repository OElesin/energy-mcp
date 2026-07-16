#!/bin/bash
# Generate and provision an API key for the Energy MCP
# Usage: ./scripts/create-key.sh <tier> [email]
# Example: ./scripts/create-key.sh pro user@example.com

set -e

TIER=${1:-pro}
EMAIL=${2:-""}
REGION="eu-west-1"
TABLE="energy-mcp-keys"

# Generate a unique key
KEY="em_$(cat /dev/urandom | LC_ALL=C tr -dc 'a-z0-9' | head -c 32)"
CREATED=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

echo "Creating API key..."
echo "  Key:   $KEY"
echo "  Tier:  $TIER"
echo "  Email: $EMAIL"
echo ""

aws dynamodb put-item \
  --region "$REGION" \
  --table-name "$TABLE" \
  --item "{
    \"api_key\": {\"S\": \"$KEY\"},
    \"tier\": {\"S\": \"$TIER\"},
    \"email\": {\"S\": \"$EMAIL\"},
    \"created_at\": {\"S\": \"$CREATED\"},
    \"active\": {\"BOOL\": true}
  }"

echo "✅ Key created successfully!"
echo ""
echo "Usage:"
echo "  curl -X POST https://energy-mcp.getbrechtai.com/mcp \\"
echo "    -H 'Authorization: Bearer $KEY' \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}'"
echo ""
echo "MCP client config:"
echo "  {\"mcpServers\":{\"energy\":{\"url\":\"https://energy-mcp.getbrechtai.com/mcp\",\"headers\":{\"Authorization\":\"Bearer $KEY\"}}}}"
