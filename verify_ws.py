#!/usr/bin/env python3
"""
WebSocket Verification Script
Tests WebSocket connection and message validation
"""

import asyncio
import json
import sys
from typing import Dict, Any

import websockets


class WebSocketVerifier:
    """Verifies WebSocket connection and message structure"""

    def __init__(self, uri: str, message_count: int = 3):
        self.uri = uri
        self.message_count = message_count
        self.received_messages = []

    async def verify_message(self, message: str) -> Dict[str, Any]:
        """Parse and verify message structure"""
        try:
            data = json.loads(message)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON: {e}")

        # Verify required fields
        required_fields = ["activeUsers", "totalUsers"]
        missing_fields = [field for field in required_fields if field not in data]

        if missing_fields:
            raise ValueError(f"Missing required fields: {missing_fields}")

        return data

    async def run(self) -> bool:
        """Run the verification test"""
        print(f"Connecting to {self.uri}...")

        try:
            async with websockets.connect(self.uri) as websocket:
                print("✓ Connected to WebSocket\n")

                for i in range(self.message_count):
                    try:
                        message = await asyncio.wait_for(
                            websocket.recv(), timeout=5.0
                        )
                        data = await self.verify_message(message)
                        self.received_messages.append(data)

                        print(f"Message {i + 1}/{self.message_count}:")
                        print(f"  Active Users: {data['activeUsers']}")
                        print(f"  Total Users:  {data['totalUsers']}")
                        print("  ✓ Verified\n")

                    except asyncio.TimeoutError:
                        print(f"✗ Timeout waiting for message {i + 1}")
                        return False
                    except ValueError as e:
                        print(f"✗ Validation error: {e}")
                        return False

                print(f"✓ SUCCESS: All {self.message_count} messages verified!")
                return True

        except websockets.exceptions.WebSocketException as e:
            print(f"✗ WebSocket error: {e}")
            return False
        except Exception as e:
            print(f"✗ Unexpected error: {e}")
            return False


async def main():
    """Main entry point"""
    WS_URI = "ws://localhost:3000/client/ws"
    MESSAGE_COUNT = 3

    verifier = WebSocketVerifier(WS_URI, MESSAGE_COUNT)
    success = await verifier.run()

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    asyncio.run(main())
