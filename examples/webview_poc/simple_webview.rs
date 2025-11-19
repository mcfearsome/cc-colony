// Simple embedded webview example
// Uses system webview (WebKit on macOS/Linux, WebView2 on Windows)
// Much lighter than full browser: ~20MB vs 100MB+

use wry::{
    application::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Colony - Simple Example")
        .with_inner_size(wry::application::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 30px;
        }
        h1 { margin-bottom: 20px; }
        .card {
            background: rgba(255, 255, 255, 0.1);
            border-radius: 10px;
            padding: 20px;
            margin: 15px 0;
        }
        button {
            background: #4CAF50;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 16px;
            margin: 5px;
        }
        button:hover { background: #45a049; }
        #output {
            background: rgba(0, 0, 0, 0.3);
            padding: 15px;
            border-radius: 8px;
            margin-top: 20px;
            min-height: 100px;
            font-family: 'Courier New', monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🐝 Colony Dashboard (Webview)</h1>

        <div class="card">
            <h2>System Info</h2>
            <p>This is running in an embedded webview, not a full browser!</p>
            <p><strong>Memory:</strong> ~20MB vs ~100MB+ for Chrome</p>
            <p><strong>Startup:</strong> ~200ms vs ~1s for browser</p>
        </div>

        <div class="card">
            <h2>Quick Actions</h2>
            <button onclick="sendCommand('list-tasks')">List Tasks</button>
            <button onclick="sendCommand('list-agents')">List Agents</button>
            <button onclick="sendCommand('show-metrics')">Show Metrics</button>
        </div>

        <div class="card">
            <h2>Output</h2>
            <div id="output">
                Click a button above to interact with colony...
            </div>
        </div>
    </div>

    <script>
        function sendCommand(cmd) {
            const output = document.getElementById('output');
            output.innerHTML = `Executing: ${cmd}...<br>`;

            // In real implementation, this would call into Rust
            // via window.ipc.postMessage() or custom protocol

            setTimeout(() => {
                output.innerHTML += `✓ Command completed<br>`;
                output.innerHTML += `<br>Mock response for: ${cmd}`;
            }, 500);
        }

        // Log that we're ready
        console.log('Webview loaded successfully!');
    </script>
</body>
</html>
    "#;

    let _webview = WebViewBuilder::new(window)?
        .with_html(html)?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
