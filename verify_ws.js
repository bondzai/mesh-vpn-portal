const WebSocket = require('ws');

function connectClient(id, onMessage) {
    const ws = new WebSocket('ws://localhost:3000/ws');
    ws.on('open', () => console.log(`Client ${id} connected`));
    ws.on('message', (data) => {
        console.log(`Client ${id} received: ${data}`);
        onMessage(JSON.parse(data));
    });
    return ws;
}

// Logic:
// 1. Client 1 connects. Should receive activeUsers >= 1.
// 2. Client 2 connects. Client 1 should receive update with activeUsers += 1.

console.log('Starting broadcast test...');

let client1Params = null;

const ws1 = connectClient(1, (data) => {
    if (!client1Params) {
        // Initial state
        client1Params = data;
        console.log('Client 1 initial state:', data);

        // Connect client 2 after a short delay
        setTimeout(() => {
            const ws2 = connectClient(2, (data2) => {
                // Client 2 just needs to connect to trigger update on Client 1
            });
        }, 500);
    } else {
        // Update received
        if (data.activeUsers === client1Params.activeUsers + 1) {
            console.log('SUCCESS: Client 1 received update from Client 2 connection');
            process.exit(0);
        }
    }
});
