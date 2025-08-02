#!/bin/bash
# Clean up test streams before running integration tests

# List and delete all test streams
nats stream ls --server nats://localhost:4222 | grep -E "(TEST_|GIT_EVENTS)" | while read stream; do
    echo "Deleting stream: $stream"
    nats stream rm "$stream" -f --server nats://localhost:4222
done

echo "Test streams cleaned up"