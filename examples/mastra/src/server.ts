import { setup } from "@rivetkit/actor";
import { aiAgent } from "./actor.js";
import { Hono } from "hono";

// Set up Rivet Actor registry
const registry = setup({ 
  use: { aiAgent }
});

const { client, serve } = registry.createServer();
const app = new Hono();

// Web interface
app.get("/", (c) => {
  return c.html(`<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rivet + Mastra - AI Chat</title>
    <style>
        * { box-sizing: border-box; }
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', 'SF Pro Display', system-ui, sans-serif; 
            margin: 0; padding: 0; 
            background: #0a0a0a; 
            color: #ffffff; 
            min-height: 100vh;
            line-height: 1.6;
        }
        .container { 
            max-width: 100vw; 
            margin: 0; 
            background: rgba(0, 0, 0, 0.4); 
            border: none;
            backdrop-filter: blur(10px);
            box-shadow: none;
            overflow: hidden;
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        .header { 
            text-align: left; 
            padding: 24px 30px; 
            background: transparent;
            border-bottom: 1px solid rgba(255, 255, 255, 0.08);
        }
        .header h1 { 
            font-size: 1.5rem; 
            font-weight: 600; 
            margin: 0; 
            color: #ffffff;
        }
        .header p { 
            color: rgba(255, 255, 255, 0.6); 
            font-size: 0.85rem; 
            margin: 2px 0 0 0; 
            font-weight: 400;
        }
        #chat { 
            flex: 1; 
            overflow-y: auto; 
            padding: 20px 30px; 
            background: transparent;
            min-height: 0;
        }
        #chat::-webkit-scrollbar { width: 6px; }
        #chat::-webkit-scrollbar-track { background: rgba(255, 255, 255, 0.05); border-radius: 3px; }
        #chat::-webkit-scrollbar-thumb { background: rgba(148, 163, 184, 0.4); border-radius: 3px; }
        #chat::-webkit-scrollbar-thumb:hover { background: rgba(148, 163, 184, 0.6); }
        .message { 
            margin: 10px 0; 
            padding: 14px 18px; 
            border-radius: 8px; 
            max-width: 55%; 
            word-wrap: break-word;
            position: relative;
            animation: fadeIn 0.2s ease-out;
            font-size: 14px;
            line-height: 1.4;
        }
        @keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
        .user { 
            background: #e2e8f0; 
            color: #374151; 
            margin-left: auto; 
            border-bottom-right-radius: 6px;
            border: none;
        }
        .assistant { 
            background: rgba(255, 255, 255, 0.03); 
            color: #d1d5db; 
            margin-right: auto;
            border: none;
            border-bottom-left-radius: 6px;
        }
        .input-section { 
            padding: 20px 30px 30px; 
            background: rgba(0, 0, 0, 0.3);
            backdrop-filter: blur(20px);
            border-top: 1px solid rgba(255, 255, 255, 0.1);
        }
        .input-group { 
            display: flex; 
            gap: 12px; 
            align-items: flex-end;
        }
        #input { 
            flex: 1; 
            padding: 14px 18px; 
            border: 1px solid rgba(255, 255, 255, 0.08); 
            border-radius: 8px; 
            font-size: 14px; 
            outline: none; 
            background: rgba(255, 255, 255, 0.04);
            color: #ffffff;
            resize: none;
            font-family: inherit;
            transition: all 0.15s ease;
        }
        #input:focus { 
            border-color: rgba(255, 255, 255, 0.15); 
            background: rgba(255, 255, 255, 0.06);
        }
        #input::placeholder { color: rgba(255, 255, 255, 0.5); }
        #send { 
            padding: 14px 20px; 
            background: #FF5C00; 
            color: #ffffff; 
            border: none; 
            cursor: pointer; 
            border-radius: 8px; 
            font-weight: 500; 
            font-size: 14px;
            transition: all 0.15s ease;
            margin-left: 10px;
        }
        #send:hover { 
            background: #e55100;
        }
        #send:disabled { 
            background: rgba(255, 255, 255, 0.1); 
            cursor: not-allowed; 
            transform: none; 
            box-shadow: none;
        }
        .user-id-header { 
            font-size: 11px; 
            color: rgba(255, 255, 255, 0.5); 
            font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
            background: rgba(255, 255, 255, 0.06);
            padding: 4px 8px;
            border-radius: 4px;
            border: 1px solid rgba(255, 255, 255, 0.1);
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="display: flex; align-items: center; justify-content: space-between; width: 100%;">
                <div>
                    <h1>AI Chat</h1>
                    <p>Persistent conversations</p>
                </div>
                <div class="user-id-header" id="userIdHeader"></div>
            </div>
        </div>
        
        <div id="chat"></div>
        
        <div class="input-section">
            <div class="input-group">
                <input type="text" id="input" placeholder="Ask me anything..." />
                <button id="send">Send</button>
            </div>
        </div>
    </div>
    
    <script>
        const userId = localStorage.getItem('userId') || 'rivet-user-' + Math.random().toString(36).slice(2, 8);
        localStorage.setItem('userId', userId);
        
        const chat = document.getElementById('chat');
        const input = document.getElementById('input');
        const send = document.getElementById('send');
        let isProcessing = false;
        
        // Show user ID in header
        document.getElementById('userIdHeader').textContent = userId;
        
        // Load conversation history from Rivet Actor
        async function loadHistory() {
            try {
                const response = await fetch(\`/chat/\${userId}/history\`);
                const data = await response.json();
                
                if (data.history && data.history.length > 0) {
                    data.history.forEach(msg => {
                        addMessage(msg.role, msg.content, false);
                    });
                }
            } catch (error) {
                console.error('Error loading history:', error);
            }
        }
        
        async function sendMessage() {
            const message = input.value.trim();
            if (!message || isProcessing) return;
            
            isProcessing = true;
            send.disabled = true;
            send.textContent = 'Processing...';
            
            addMessage('user', message);
            input.value = '';
            
            try {
                const response = await fetch('/chat', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ userId, message })
                });
                const data = await response.json();
                
                if (data.error) {
                    addMessage('assistant', 'Error: ' + data.error);
                } else {
                    addMessage('assistant', data.response);
                }
            } catch (error) {
                addMessage('assistant', 'Network Error: ' + error.message);
            }
            
            isProcessing = false;
            send.disabled = false;
            send.textContent = 'Send';
        }
        
        function addMessage(role, content, scroll = true) {
            const div = document.createElement('div');
            div.className = \`message \${role}\`;
            
            const messageContent = document.createElement('div');
            messageContent.className = 'message-content';
            messageContent.textContent = content;
            
            div.appendChild(messageContent);
            chat.appendChild(div);
            
            if (scroll) {
                chat.scrollTop = chat.scrollHeight;
            }
        }
        
        send.addEventListener('click', sendMessage);
        input.addEventListener('keypress', (e) => {
            if (e.key === 'Enter' && !isProcessing) sendMessage();
        });
        
        // Load Rivet Actor history when page loads
        loadHistory();
    </script>
</body>
</html>`);
});

// Chat with AI agent
app.post("/chat", async (c) => {
  const { message, userId } = await c.req.json();
  
  // Get or create persistent AI agent for this user
  const actor = await client.aiAgent.getOrCreate([userId]);
  const result = await actor.chat(message);
  
  return c.json(result);
});

// Get conversation history
app.get("/chat/:userId/history", async (c) => {
  const { userId } = c.req.param();
  const actor = await client.aiAgent.getOrCreate([userId]);
  const result = await actor.getHistory();
  
  return c.json(result);
});

// Clear conversation
app.delete("/chat/:userId", async (c) => {
  const { userId } = c.req.param();
  const actor = await client.aiAgent.getOrCreate([userId]);
  const result = await actor.clear();
  
  return c.json(result);
});

// Start server with Rivet integration
serve(app);