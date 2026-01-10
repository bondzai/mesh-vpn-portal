const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:3000/ws');

ws.on('open', function open() {
    console.log('connected');
});

let messageCount = 0;

ws.on('message', function message(data) {
    console.log('received: %s', data);
    const parsed = JSON.parse(data);

    if (parsed.activeUsers !== undefined && parsed.totalUsers !== undefined) {
        console.log('Structure verified');
    } else {
        console.error('Invalid structure');
        process.exit(1);
    }

    messageCount++;
    if (messageCount >= 3) {
        console.log('Success');
        process.exit(0);
    }
});

ws.on('error', console.error);
