# WebSocket Tests

This directory contains verification scripts for testing the WebSocket functionality of the counter service.

## Available Tests

### `verify_ws.js` (JavaScript/Node.js)
Tests WebSocket broadcast functionality by:
1. Connecting client 1 and capturing initial state
2. Connecting client 2 after a delay
3. Verifying client 1 receives broadcast update with incremented user count

**Run:**
```bash
make verify-js
```

### `verify_ws.py` (Python)
Tests WebSocket connection and message structure by:
1. Connecting to the WebSocket endpoint
2. Receiving and validating multiple messages
3. Verifying required fields (`activeUsers`, `totalUsers`)

**Run:**
```bash
make verify-py
```

## Run All Tests

```bash
make test-ws
```

## Requirements

- **JavaScript**: Node.js with `ws` package (`npm install ws`)
- **Python**: Python 3.7+ with `websockets` package (`pip install websockets`)

## Notes

- WebSocket endpoint: `ws://localhost:3000/client/ws`
- Both scripts have 5-second timeout protection
- Exit code 0 = success, 1 = failure
