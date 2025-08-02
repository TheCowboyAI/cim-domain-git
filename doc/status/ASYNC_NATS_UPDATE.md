# async-nats 0.42 Update Summary

Copyright 2025 Cowboy AI, LLC.

## Overview

Successfully updated cim-domain-git from async-nats 0.33 to 0.42, achieving **0 compilation errors** in the library code.

## Key Changes Made

### 1. API Updates
- Removed `max_reconnects` from ConnectOptions (no longer available)
- Changed `max_msgs_per_subject` to `max_messages` in stream config
- Updated field types: `u64` to `i64` for max_messages
- Fixed `DeliverPolicy::ByStartSequence` to use struct syntax with `start_sequence` field

### 2. Stream Handling
- Added `StreamExt` imports for using `.next()` on Subscribers
- Changed `first_seq`/`last_seq` to `first_sequence`/`last_sequence`
- Made some methods take `&mut self` due to new borrowing requirements

### 3. Header Values
- Fixed header insertion to use owned strings instead of references
- Changed from `&str` to `String` for header values

### 4. Message Handling
- Removed `message.client` - now pass Client separately to handlers
- Updated message reply handling to use client directly

### 5. Error Handling
- Simplified `NatsError::Timeout` to not take a String parameter
- Updated error conversions for new async-nats error types

### 6. TLS Configuration
- Simplified TLS handling - async-nats 0.42 handles TLS automatically for tls:// URLs
- Removed manual certificate configuration code

## Results

- **Library Code**: ✅ 0 errors, 66 warnings (mostly missing documentation)
- **Tests**: ❌ Need updates for API changes
- **Examples**: ❌ Need updates for API changes

## Next Steps

1. Update tests for new async-nats API
2. Update examples for new async-nats API
3. Add missing documentation to reduce warnings
4. Consider re-enabling OpenTelemetry tracing once system dependencies are resolved

## Benefits of 0.42

- More stable API
- Better performance
- Improved error handling
- Simpler TLS configuration
- Better async/await integration