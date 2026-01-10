import asyncio
import websockets
import json

async def test():
    uri = "ws://localhost:3000/ws"
    async with websockets.connect(uri) as websocket:
        print("Connected to WebSocket")
        for i in range(3):
            message = await websocket.recv()
            print(f"Received: {message}")
            data = json.loads(message)
            assert "activeUsers" in data
            assert "totalUsers" in data
            print("Verified message verification")
        print("Success")

if __name__ == "__main__":
    asyncio.run(test())
