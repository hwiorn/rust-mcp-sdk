// Import the generated JS glue file. build.sh copies it to the project root.
import init, { WasmClient } from './mcp_wasm_client.js';

async function main() {
    await init();

    const connectBtn = document.getElementById('connect-btn');
    const disconnectBtn = document.getElementById('disconnect-btn');
    const listToolsBtn = document.getElementById('list-tools-btn');
    const serverUrlInput = document.getElementById('server-url');
    const statusDiv = document.getElementById('status');
    const toolsList = document.getElementById('tools-list');
    const logs = document.getElementById('logs');

    let client = null;

    function log(message) {
        logs.textContent += message + '\n';
    }

    connectBtn.addEventListener('click', async () => {
        const url = serverUrlInput.value;
        if (!url) {
            log('Please enter a server URL.');
            return;
        }

        log(`Connecting to ${url}...`);
        try {
            client = new WasmClient();
            await client.connect(url);
            statusDiv.textContent = `Connected to ${url}`;
            connectBtn.disabled = true;
            disconnectBtn.disabled = false;
            listToolsBtn.disabled = false;
        } catch (e) {
            log(`Error: ${e.message}`);
            if (e.code) log(`  Code: ${e.code}`);
            if (e.data) log(`  Data: ${JSON.stringify(e.data)}`);
            client = null;
        }
    });

    disconnectBtn.addEventListener('click', () => {
        client = null;
        statusDiv.textContent = 'Not connected';
        connectBtn.disabled = false;
        disconnectBtn.disabled = true;
        listToolsBtn.disabled = true;
        toolsList.innerHTML = '';
        log('Disconnected.');
    });

    listToolsBtn.addEventListener('click', async () => {
        if (!client) return;
        log('Listing tools...');
        try {
            const tools = await client.list_tools();
            toolsList.innerHTML = '';
            if (tools.length === 0) {
                toolsList.innerHTML = '<li>No tools found.</li>';
            } else {
                for (const tool of tools) {
                    const li = document.createElement('li');
                    li.textContent = tool.name;
                    toolsList.appendChild(li);
                }
            }
        } catch (e) {
            log(`Error: ${e.message}`);
        }
    });
}

main();
