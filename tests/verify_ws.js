const WebSocket = require('ws');

/**
 * WebSocket Broadcast Verification Script
 * Tests that WebSocket broadcasts work correctly when clients connect/disconnect
 */

const WS_URL = 'ws://localhost:3000/client/ws';
const CONNECTION_DELAY = 500; // ms

class WebSocketClient {
    constructor(id, url) {
        this.id = id;
        this.url = url;
        this.ws = null;
        this.initialState = null;
    }

    connect(onMessage) {
        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(this.url);

            this.ws.on('open', () => {
                console.log(`✓ Client ${this.id} connected`);
                resolve();
            });

            this.ws.on('message', (data) => {
                try {
                    const parsed = JSON.parse(data);
                    console.log(`  Client ${this.id} received:`, parsed);
                    onMessage(parsed);
                } catch (err) {
                    console.error(`✗ Client ${this.id} parse error:`, err.message);
                }
            });

            this.ws.on('error', (err) => {
                console.error(`✗ Client ${this.id} error:`, err.message);
                reject(err);
            });

            this.ws.on('close', () => {
                console.log(`  Client ${this.id} disconnected`);
            });
        });
    }

    close() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

async function runTest() {
    console.log('Starting WebSocket broadcast test...\n');

    const client1 = new WebSocketClient(1, WS_URL);
    let client2;

    try {
        // Connect client 1 and wait for initial state
        await client1.connect((data) => {
            if (!client1.initialState) {
                client1.initialState = data;
                console.log('  Initial active users:', data.activeUsers);

                // Connect client 2 after delay
                setTimeout(async () => {
                    client2 = new WebSocketClient(2, WS_URL);
                    await client2.connect(() => { });
                }, CONNECTION_DELAY);
            } else {
                // Check if broadcast received after client 2 connected
                const expectedUsers = client1.initialState.activeUsers + 1;
                if (data.activeUsers === expectedUsers) {
                    console.log('\n✓ SUCCESS: Broadcast working correctly!');
                    console.log(`  Active users updated from ${client1.initialState.activeUsers} to ${data.activeUsers}`);

                    // Clean up
                    client1.close();
                    if (client2) client2.close();
                    process.exit(0);
                } else {
                    console.error(`✗ FAIL: Expected ${expectedUsers} users, got ${data.activeUsers}`);
                    process.exit(1);
                }
            }
        });

        // Timeout after 5 seconds
        setTimeout(() => {
            console.error('\n✗ TIMEOUT: Test did not complete in time');
            client1.close();
            if (client2) client2.close();
            process.exit(1);
        }, 5000);

    } catch (err) {
        console.error('\n✗ Test failed:', err.message);
        process.exit(1);
    }
}

runTest();
