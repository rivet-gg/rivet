import dgram from 'dgram';

export function createAndStartUdpServer() {
	// Get port from environment
	const portEnv = process.env.PORT_UDP;

	if (portEnv) {
		// Create a UDP socket
		const udpServer = dgram.createSocket('udp4');

		// Listen for incoming messages
		udpServer.on('message', (msg, rinfo) => {
			console.log(`UDP server received: ${msg} from ${rinfo.address}:${rinfo.port}`);

			// Echo the message back to the sender
			udpServer.send(msg, rinfo.port, rinfo.address, (err) => {
				if (err) console.error('Failed to send UDP response:', err);
			});
		});

		// Handle errors
		udpServer.on('error', (err) => {
			console.error('UDP server error:', err);
			udpServer.close();
		});


		const port2 = Number.parseInt(portEnv);

		udpServer.bind(port2, () => {
			console.log(`UDP echo server running on port ${port2}`);
		});
	} else {
		console.warn("missing PORT_UDP");
	}
}