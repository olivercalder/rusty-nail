#!/bin/sh

SERVER="$1"
PORT="$2"
IMAGE="$3"
THUMBNAIL="$4"

USAGE="USAGE: $0 SERVER PORT FILENAME"

[ -n "$SERVER" ] || { echo "ERROR: missing required arg: SERVER"; echo "$USAGE"; exit 1; }
[ -n "$PORT" ] || { echo "ERROR: missing required arg: PORT"; echo "$USAGE"; exit 1; }
[ -n "$IMAGE" ] || { echo "ERROR: missing required arg: IMAGE"; echo "$USAGE"; exit 1; }
[ -f "$IMAGE" ] || { echo "ERROR: IMAGE file not found: '$IMAGE'"; exit 1; }
[ -n "$THUMBNAIL" ] || { echo "ERROR: missing required arg: THUMBNAIL"; echo "$USAGE"; exit 1; }

du -b "$IMAGE" | awk -F ' ' '{print $1}' | netcat "$SERVER" "$PORT"
netcat "$SERVER" "$PORT" < "$IMAGE" > "$THUMBNAIL"
