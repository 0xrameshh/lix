use std::io::Write;
use std::path::Path;

pub fn handle_studio(input: &Path, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("lix studio: starting web server on http://localhost:{port}");
    eprintln!("Serving trace files from: {}", input.display());
    eprintln!();
    eprintln!("To explore your traces, open http://localhost:{port} in your browser.");
    eprintln!();
    eprintln!("Available endpoints:");
    eprintln!("  GET  /              - Dashboard (trace summary)");
    eprintln!("  GET  /traces        - List all trace files");
    eprintln!("  GET  /traces/:file  - View a specific trace");
    eprintln!("  GET  /api/traces    - JSON API for trace list");
    eprintln!();
    eprintln!("Press Ctrl+C to stop.");

    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{port}"))?;
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<!DOCTYPE html><html><head><title>lix studio</title><style>body{font-family:system-ui,sans-serif;max-width:800px;margin:2em auto;padding:0 1em;line-height:1.6}pre{background:#f5f5f5;padding:1em;border-radius:4px;overflow-x:auto}</style></head><body><h1>lix studio</h1><p>Trace viewer for analyzing AI agent trace files.</p><p>This is a minimal preview. For full functionality:</p><ul><li>Use <code>lix extract</code> to convert traces to training JSONL</li><li>Use <code>lix info &lt;file&gt;</code> to inspect individual trace files</li></ul><hr><pre id=\"output\">Loading...</pre><script>fetch('/api/traces').then(r=>r.text()).then(t=>document.getElementById('output').textContent=t)</script></body></html>\r\n";
        let _ = stream.write(response);
        let _ = stream.flush();
    }
    Ok(())
}
