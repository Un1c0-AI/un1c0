#!/bin/bash
set -e

# AppRole Accessor Cleanup Script
# Revokes expired secret_id accessors and cleans up old issuance events

VAULT_ADDR="${VAULT_ADDR:-http://127.0.0.1:8200}"
VAULT_TOKEN="${VAULT_TOKEN:-root-token}"
APPROLE_NAME="${APPROLE_NAME:-master-key-approle}"
MAX_ACCESSORS="${MAX_ACCESSORS:-10}"
MAX_EVENTS="${MAX_EVENTS:-100}"

echo "AppRole Accessor Cleanup"
echo "========================"
echo "Vault: $VAULT_ADDR"
echo "AppRole: $APPROLE_NAME"
echo "Max Accessors to keep: $MAX_ACCESSORS"
echo "Max Events to keep: $MAX_EVENTS"
echo ""

# Function to call Vault API
vault_api() {
    local method="$1"
    local path="$2"
    local data="$3"
    
    if [ -n "$data" ]; then
        curl -s -X "$method" \
            -H "X-Vault-Token: $VAULT_TOKEN" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$VAULT_ADDR/v1/$path"
    else
        curl -s -X "$method" \
            -H "X-Vault-Token: $VAULT_TOKEN" \
            "$VAULT_ADDR/v1/$path"
    fi
}

# Get current accessors
echo "Fetching stored accessors..."
ACCESSOR_DATA=$(vault_api GET "kv/data/approle_accessors")
ACCESSOR_COUNT=$(echo "$ACCESSOR_DATA" | jq -r '.data.data.accessors // [] | length')
echo "Found $ACCESSOR_COUNT stored accessors"

if [ "$ACCESSOR_COUNT" -gt "$MAX_ACCESSORS" ]; then
    echo "Accessor count ($ACCESSOR_COUNT) exceeds max ($MAX_ACCESSORS)"
    
    # Get all accessors
    ACCESSORS=$(echo "$ACCESSOR_DATA" | jq -r '.data.data.accessors // []')
    
    # Keep only the last MAX_ACCESSORS
    ACCESSORS_TO_KEEP=$(echo "$ACCESSORS" | jq ".[-${MAX_ACCESSORS}:]")
    ACCESSORS_TO_REVOKE=$(echo "$ACCESSORS" | jq ".[:-${MAX_ACCESSORS}]")
    
    # Revoke old accessors
    REVOKE_COUNT=$(echo "$ACCESSORS_TO_REVOKE" | jq 'length')
    echo "Revoking $REVOKE_COUNT old accessors..."
    
    echo "$ACCESSORS_TO_REVOKE" | jq -r '.[] | .accessor' | while read -r accessor; do
        if [ -n "$accessor" ] && [ "$accessor" != "null" ]; then
            echo "  Revoking accessor: $accessor"
            vault_api POST "auth/approle/role/$APPROLE_NAME/secret-id-accessor/destroy" \
                "{\"secret_id_accessor\":\"$accessor\"}" > /dev/null || echo "    Failed to revoke $accessor"
        fi
    done
    
    # Update stored accessors
    echo "Updating stored accessors to keep only last $MAX_ACCESSORS..."
    vault_api POST "kv/data/approle_accessors" \
        "{\"data\":{\"accessors\":$ACCESSORS_TO_KEEP}}" > /dev/null
    
    echo "✅ Cleaned up $REVOKE_COUNT accessors"
else
    echo "✅ Accessor count within limits, no cleanup needed"
fi

echo ""
echo "Fetching issuance events..."
EVENT_DATA=$(vault_api GET "kv/data/issuance_events")
EVENT_COUNT=$(echo "$EVENT_DATA" | jq -r '.data.data.events // [] | length')
echo "Found $EVENT_COUNT issuance events"

if [ "$EVENT_COUNT" -gt "$MAX_EVENTS" ]; then
    echo "Event count ($EVENT_COUNT) exceeds max ($MAX_EVENTS)"
    
    # Get all events
    EVENTS=$(echo "$EVENT_DATA" | jq -r '.data.data.events // []')
    
    # Keep only the last MAX_EVENTS
    EVENTS_TO_KEEP=$(echo "$EVENTS" | jq ".[-${MAX_EVENTS}:]")
    EVENTS_TO_DELETE=$(( EVENT_COUNT - MAX_EVENTS ))
    
    echo "Removing $EVENTS_TO_DELETE old events..."
    vault_api POST "kv/data/issuance_events" \
        "{\"data\":{\"events\":$EVENTS_TO_KEEP}}" > /dev/null
    
    echo "✅ Cleaned up $EVENTS_TO_DELETE events"
else
    echo "✅ Event count within limits, no cleanup needed"
fi

echo ""
echo "Cleanup completed successfully!"
