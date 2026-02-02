pub fn generate_dashboard_html(peer_id: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Erynoa Diagnostics</title>
    <style>
        :root {{
            /* Shadcn Zinc Light Theme */
            --bg: #ffffff;
            --fg: #09090b;
            --card: #ffffff;
            --card-fg: #09090b;
            --border: #e4e4e7;
            --input: #e4e4e7;
            --muted: #f4f4f5;
            --muted-fg: #71717a;

            /* Semantics */
            --primary: #18181b;
            --primary-fg: #fafafa;

            --success: #16a34a; --success-bg: #dcfce7;
            --warning: #ca8a04; --warning-bg: #fef9c3;
            --error: #dc2626;   --error-bg: #fee2e2;
            --info: #2563eb;    --info-bg: #dbeafe;

            --radius: 0.5rem;
            --font-sans: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            --font-mono: "JetBrains Mono", "SF Mono", monospace;
        }}

        * {{ box-sizing: border-box; }}

        body {{
            background: var(--bg);
            color: var(--fg);
            font-family: var(--font-sans);
            margin: 0;
            padding: 24px;
            font-size: 14px;
            line-height: 1.5;
            -webkit-font-smoothing: antialiased;
        }}

        /* --- Grid Layout --- */
        .dashboard {{
            display: grid;
            grid-template-columns: 320px 1fr 380px;
            gap: 24px;
            max-width: 1600px;
            margin: 0 auto;
            align-items: start;
        }}

        @media (max-width: 1400px) {{ .dashboard {{ grid-template-columns: 300px 1fr; }} }}
        @media (max-width: 1000px) {{ .dashboard {{ grid-template-columns: 1fr; }} }}

        .col {{ display: flex; flex-direction: column; gap: 24px; }}

        /* --- Card Component --- */
        .card {{
            background: var(--card);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
            overflow: hidden;
            display: flex; flex-direction: column;
        }}

        .card-head {{
            padding: 16px;
            border-bottom: 1px solid var(--border);
            display: flex; justify-content: space-between; align-items: center;
            background: #fafafa; /* Minimal contrast header */
        }}

        .card-title {{
            font-size: 13px; font-weight: 600; color: var(--fg);
            letter-spacing: -0.01em; display: flex; align-items: center; gap: 8px;
        }}

        .card-body {{ padding: 0; }}
        .padded {{ padding: 16px; }}

        /* --- Typography --- */
        .mono {{ font-family: var(--font-mono); letter-spacing: -0.02em; }}
        .text-sm {{ font-size: 13px; }}
        .text-xs {{ font-size: 12px; }}
        .text-muted {{ color: var(--muted-fg); }}
        .font-medium {{ font-weight: 500; }}
        .font-bold {{ font-weight: 600; }}

        /* --- KPI Strip --- */
        .kpi-grid {{
            display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px;
            max-width: 1600px; margin: 0 auto 24px;
        }}

        .kpi {{
            background: var(--card); border: 1px solid var(--border);
            border-radius: var(--radius); padding: 16px 20px;
            display: flex; flex-direction: column; justify-content: center;
            box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
        }}

        .kpi-label {{ font-size: 11px; font-weight: 500; color: var(--muted-fg); text-transform: uppercase; letter-spacing: 0.05em; }}
        .kpi-val {{ font-size: 24px; font-weight: 700; color: var(--fg); margin-top: 4px; letter-spacing: -0.03em; }}
        .kpi-sub {{ font-size: 12px; color: var(--muted-fg); margin-top: 4px; display: flex; align-items: center; gap: 6px; }}

        /* --- Data Rows --- */
        .row {{
            display: flex; justify-content: space-between; align-items: center;
            padding: 10px 16px; border-bottom: 1px solid var(--border);
            font-size: 13px;
        }}
        .row:last-child {{ border-bottom: none; }}
        .val {{ font-family: var(--font-mono); font-weight: 500; color: var(--fg); }}

        /* Flash Animation for Updates */
        .flash {{ animation: flash-anim 0.4s ease-out; }}
        @keyframes flash-anim {{ 0% {{ background-color: rgba(24, 24, 27, 0.1); }} 100% {{ background-color: transparent; }} }}

        /* --- Badges --- */
        .badge {{
            display: inline-flex; align-items: center; padding: 2px 8px;
            border-radius: 6px; font-size: 11px; font-weight: 600;
            border: 1px solid transparent; white-space: nowrap;
        }}
        .b-outline {{ border-color: var(--border); color: var(--fg); background: var(--bg); }}
        .b-green {{ background: var(--success-bg); color: #14532d; border-color: #bbf7d0; }}
        .b-yellow {{ background: var(--warning-bg); color: #713f12; border-color: #fde68a; }}
        .b-red {{ background: var(--error-bg); color: #7f1d1d; border-color: #fecaca; }}
        .b-dark {{ background: var(--primary); color: var(--primary-fg); }}

        /* --- Canvas Chart --- */
        .chart-container {{ position: relative; height: 100px; width: 100%; background: #fafafa; border-bottom: 1px solid var(--border); }}
        canvas {{ display: block; width: 100%; height: 100%; }}

        /* --- Layers --- */
        .layer {{ border: 1px solid var(--border); border-radius: 6px; margin: 12px 16px; overflow: hidden; }}
        .layer-h {{
            background: #fafafa; padding: 8px 12px; font-size: 12px; font-weight: 600;
            display: flex; justify-content: space-between; border-bottom: 1px solid var(--border);
        }}
        .check {{ padding: 6px 12px; display: flex; gap: 10px; font-size: 12px; align-items: flex-start; }}
        .dot {{ width: 6px; height: 6px; border-radius: 50%; margin-top: 6px; flex-shrink: 0; }}

        /* --- Peer Table --- */
        .p-table {{ width: 100%; border-collapse: collapse; font-size: 12px; }}
        .p-table th {{ text-align: left; padding: 10px 16px; color: var(--muted-fg); font-weight: 500; background: #fafafa; border-bottom: 1px solid var(--border); }}
        .p-table td {{ padding: 8px 16px; border-bottom: 1px solid var(--border); font-family: var(--font-mono); color: var(--fg); }}
        .p-table tr:last-child td {{ border-bottom: none; }}

        /* --- Logs --- */
        .log-item {{
            padding: 10px 16px; border-bottom: 1px solid var(--border);
            display: flex; gap: 12px; align-items: flex-start;
            animation: slide-in 0.2s ease-out;
        }}
        @keyframes slide-in {{ from {{ opacity: 0; transform: translateY(-4px); }} to {{ opacity: 1; transform: translateY(0); }} }}

        /* Utils */
        .copy {{ cursor: pointer; transition: all 0.2s; }}
        .copy:hover {{ background: var(--muted); color: var(--primary); }}

        .pulse {{ width: 6px; height: 6px; background: var(--success); border-radius: 50%; box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7); animation: pulse 2s infinite; }}
        @keyframes pulse {{ 0% {{ box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.4); }} 70% {{ box-shadow: 0 0 0 6px rgba(34, 197, 94, 0); }} 100% {{ box-shadow: 0 0 0 0 rgba(34, 197, 94, 0); }} }}

        /* Toast */
        .toast {{
            position: fixed; bottom: 24px; right: 24px;
            background: var(--primary); color: var(--primary-fg);
            padding: 10px 16px; border-radius: 6px; font-size: 13px; font-weight: 500;
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
            transform: translateY(20px); opacity: 0; pointer-events: none;
            transition: all 0.2s ease;
        }}
        .toast.show {{ transform: translateY(0); opacity: 1; }}
    </style>
</head>
<body>

    <div style="max-width:1600px; margin:0 auto 24px; display:flex; justify-content:space-between; align-items:center;">
        <div style="display:flex; align-items:center; gap:12px;">
            <div style="width:36px; height:36px; background:var(--primary); border-radius:8px; color:white; display:grid; place-items:center; font-weight:700; font-size:18px;">E</div>
            <div>
                <h1 style="margin:0; font-size:16px; font-weight:600; letter-spacing:-0.01em;">Erynoa Diagnostics</h1>
                <div style="display:flex; gap:8px; margin-top:2px;">
                    <span class="badge b-outline mono copy" onclick="copy('{peer_id}')" title="Copy Peer ID">{peer_id}</span>
                </div>
            </div>
        </div>
        <div style="text-align:right;">
            <span class="badge b-green" id="status-badge">SYSTEM OPERATIONAL</span>
            <div class="text-xs text-muted mono" style="margin-top:6px;">
                <span id="uptime">00:00:00</span> • <span style="color:var(--success)">●</span> LIVE
            </div>
        </div>
    </div>

    <div class="kpi-grid">
        <div class="kpi">
            <span class="kpi-label">Active Peers</span>
            <span class="kpi-val" id="kpi-peers">0</span>
            <span class="kpi-sub"><span style="color:var(--success)">●</span> <span id="kpi-dht">0</span> discovered</span>
        </div>
        <div class="kpi">
            <span class="kpi-label">Latency (Avg)</span>
            <span class="kpi-val" id="kpi-lat">0<span style="font-size:16px; color:var(--muted-fg)">ms</span></span>
            <span class="kpi-sub">P95: <span id="kpi-p95" class="mono">0</span> ms</span>
        </div>
        <div class="kpi">
            <span class="kpi-label">Throughput</span>
            <span class="kpi-val" id="kpi-bw">0<span style="font-size:16px; color:var(--muted-fg)">/s</span></span>
            <span class="kpi-sub">In: <span id="kpi-in">0</span> • Out: <span id="kpi-out">0</span></span>
        </div>
        <div class="kpi">
            <span class="kpi-label">Health Score</span>
            <span class="kpi-val" id="kpi-health" style="color:var(--success)">100%</span>
            <span class="kpi-sub"><span id="kpi-checks">0</span> checks passed</span>
        </div>
    </div>

    <div class="dashboard">

        <div class="col">
            <div class="card" style="height:100%; max-height:800px;">
                <div class="card-head">
                    <span class="card-title">Infrastructure Layers</span>
                    <span class="badge b-outline">Stack</span>
                </div>
                <div class="card-body" id="layers" style="overflow-y:auto; padding-bottom:12px;"></div>
            </div>
        </div>

        <div class="col">
            <div class="card">
                <div class="card-head">
                    <span class="card-title">Live Traffic (60s)</span>
                    <div class="pulse"></div>
                </div>
                <div class="chart-container">
                    <canvas id="chart"></canvas>
                </div>
            </div>

            <div class="card">
                <div class="card-head"><span class="card-title">Network Transport</span></div>
                <div class="card-body">
                    <div class="row"><span class="text-muted">Addresses</span><span class="val text-xs" id="v-addr">-</span></div>
                    <div class="row"><span class="text-muted">Connections</span><span class="val"><span id="v-cin">0</span> In / <span id="v-cout">0</span> Out</span></div>
                    <div class="row"><span class="text-muted">Errors</span><span class="val" id="v-err" style="color:var(--error)">0</span></div>
                </div>
            </div>

            <div class="card">
                <div class="card-head"><span class="card-title">DHT & Discovery</span></div>
                <div class="card-body">
                    <div class="row"><span class="text-muted">Routing Table</span><span class="val"><span id="v-dht">0</span> Peers</span></div>
                    <div class="row"><span class="text-muted">mDNS</span><span class="val"><span id="v-mdns">0</span> Local</span></div>
                    <div class="row"><span class="text-muted">Records</span><span class="val" id="v-recs">0</span></div>
                </div>
            </div>

            <div class="card">
                <div class="card-head"><span class="card-title">NAT & Privacy</span></div>
                <div class="card-body">
                    <div class="row"><span class="text-muted">NAT Status</span><span class="badge b-outline" id="v-nat">-</span></div>
                    <div class="row"><span class="text-muted">DCUTR Rate</span><span class="val" id="v-dcutr">0%</span></div>
                    <div class="row"><span class="text-muted">Onion Circuits</span><span class="val" id="v-onion">0</span></div>
                </div>
            </div>
        </div>

        <div class="col">
            <div class="card" style="flex:1; max-height:450px;">
                <div class="card-head">
                    <span class="card-title">Swarm</span>
                    <span class="badge b-dark" id="badge-peers">0</span>
                </div>
                <div class="card-body" style="overflow-y:auto; padding:0;">
                    <table class="p-table">
                        <thead><tr><th>ID / Addr</th><th style="text-align:right">RTT</th><th style="text-align:right">Flags</th></tr></thead>
                        <tbody id="p-rows"></tbody>
                    </table>
                </div>
            </div>

            <div class="card" style="height:400px;">
                <div class="card-head">
                    <span class="card-title">Event Log</span>
                </div>
                <div class="card-body" id="logs" style="overflow-y:auto; padding:0; background:#fafafa;"></div>
            </div>
        </div>
    </div>

    <div id="toast" class="toast">Copied to clipboard</div>

    <script>
        const $ = id => document.getElementById(id);
        const fmtB = b => !+b ? '0 B' : `${{parseFloat((b/Math.pow(1024, Math.floor(Math.log(b)/Math.log(1024)))).toFixed(1))}} ${{['B','K','M','G'][Math.floor(Math.log(b)/Math.log(1024))]}}`;
        const fmtT = s => new Date(s * 1000).toISOString().substr(11, 8);

        // --- Optimized Charting (Canvas) ---
        const canvas = $('chart');
        const ctx = canvas.getContext('2d');
        const data = {{ in: new Array(60).fill(0), out: new Array(60).fill(0) }};

        function resizeChart() {{
            const rect = canvas.parentElement.getBoundingClientRect();
            const dpr = window.devicePixelRatio || 1;
            canvas.width = rect.width * dpr;
            canvas.height = rect.height * dpr;
            ctx.scale(dpr, dpr);
            canvas.style.width = `${{rect.width}}px`;
            canvas.style.height = `${{rect.height}}px`;
        }}
        window.addEventListener('resize', resizeChart);
        resizeChart(); // Init

        function drawChart() {{
            const w = canvas.width / (window.devicePixelRatio || 1);
            const h = canvas.height / (window.devicePixelRatio || 1);
            ctx.clearRect(0, 0, w, h);

            const max = Math.max(1024, ...data.in, ...data.out) * 1.2;
            const step = w / 59;

            const paint = (arr, color, fill) => {{
                ctx.beginPath();
                ctx.moveTo(0, h - (arr[0]/max * h));
                arr.forEach((v, i) => ctx.lineTo(i * step, h - (v/max * h)));
                ctx.lineWidth = 1.5; ctx.strokeStyle = color; ctx.stroke();
                if(fill) {{
                    ctx.lineTo(w, h); ctx.lineTo(0, h);
                    ctx.fillStyle = fill; ctx.fill();
                }}
            }};

            paint(data.in, '#2563eb', 'rgba(37, 99, 235, 0.05)'); // Blue In
            paint(data.out, '#16a34a', 'rgba(22, 163, 74, 0.05)'); // Green Out
        }}

        // --- Core Updater ---
        function setVal(id, txt) {{
            const el = $(id);
            if(!el) return;
            const s = String(txt);
            if(el.innerText !== s) {{
                el.innerText = s;
                el.classList.remove('flash');
                void el.offsetWidth;
                el.classList.add('flash');
            }}
        }}

        function renderLayers(layers) {{
            if(!layers) return;
            let h = '';
            layers.forEach(l => {{
                let st = l.overall_status;
                let col = st==='healthy'?'var(--success)':st==='degraded'?'#ca8a04':'#dc2626';
                h += `<div class="layer"><div class="layer-h"><span>${{l.layer_name.split('(')[0]}}</span><span style="color:${{col}}">${{st.toUpperCase()}}</span></div>`;
                l.checks.forEach(c => {{
                    let dot = c.status==='healthy'?'#22c55e':c.status==='degraded'?'#ca8a04':'#dc2626';
                    let v = c.metric_value != null ? (c.metric_value%1!==0?c.metric_value.toFixed(1):c.metric_value) : '';
                    h += `<div class="check"><div class="dot" style="background:${{dot}}"></div><div style="flex:1"><div class="font-bold text-xs">${{c.name}}</div><div class="text-xs text-muted">${{v ? `<span class="val">${{v}}${{c.metric_unit||''}}</span> ` : ''}}${{c.message}}</div></div></div>`;
                }});
                h += `</div>`;
            }});
            $('layers').innerHTML = h;
        }}

        function renderPeers(peers) {{
            const el = $('p-rows');
            $('badge-peers').innerText = peers.length;
            if(!peers.length) return el.innerHTML = '<tr><td colspan="3" style="text-align:center; padding:30px; color:#a1a1aa">No active peers</td></tr>';

            el.innerHTML = peers.map(p => {{
                let rc = p.ping_rtt_ms < 60 ? 'var(--success)' : p.ping_rtt_ms < 200 ? '#ca8a04' : '#dc2626';
                let f=[]; if(p.is_relayed)f.push('R'); if(p.in_gossip_mesh)f.push('G'); if(p.in_kademlia)f.push('K');
                return `<tr>
                    <td><div class="copy font-medium" onclick="copy('${{p.peer_id}}')">${{p.peer_id.substring(0,8)}}...</div><div class="text-xs text-muted">${{p.address||'-'}}</div></td>
                    <td style="text-align:right; font-family:var(--font-mono); color:${{rc}}">${{p.ping_rtt_ms}}ms</td>
                    <td style="text-align:right;">${{f.map(x=>`<span class="badge b-outline" style="font-size:9px; padding:0 4px">${{x}}</span>`).join(' ')}}</td>
                </tr>`;
            }}).join('');
        }}

        const seenLog = new Set();
        function renderLog(events) {{
            if(!events) return;
            const c = $('logs');
            events.forEach(e => {{
                if(seenLog.has(e.id)) return;
                seenLog.add(e.id);
                let ic='ℹ️', t=e.event_type.toLowerCase();
                if(e.severity==='error'||t.includes('fail')) ic='🔥';
                else if(e.severity==='warning') ic='⚠️';
                else if(t.includes('success')) ic='✅';
                else if(t.includes('gossip')) ic='📨';

                const html = `<div class="log-item">
                    <div style="font-size:14px">${{ic}}</div>
                    <div style="flex:1">
                        <div style="font-size:12px; line-height:1.4; color:#3f3f46">${{e.message}}</div>
                        <div class="mono text-xs text-muted" style="margin-top:2px; font-size:10px;">
                            ${{e.timestamp.split('T')[1].split('.')[0]}} • ${{e.event_type}}
                        </div>
                    </div>
                </div>`;
                c.insertAdjacentHTML('afterbegin', html);
                if(c.children.length > 50) c.lastElementChild.remove();
            }});
        }}

        function update(d) {{
            const m=d.metrics||{{}}, s=d.swarm||{{}}, h=d.health||{{}};

            // Chart
            data.in.shift(); data.in.push(m.bytes_per_second_in||0);
            data.out.shift(); data.out.push(m.bytes_per_second_out||0);
            drawChart();

            // Status
            $('status-badge').innerText = (h.status||'OK').toUpperCase();
            $('uptime').innerText = fmtT(s.uptime_secs||0);

            // KPIs
            setVal('kpi-peers', s.connected_peers_count??0);
            setVal('kpi-dht', s.mdns_discovered_count??0);
            setVal('kpi-lat', (s.avg_ping_ms||0).toFixed(0));
            setVal('kpi-p95', (s.max_ping_ms||0).toFixed(0));
            setVal('kpi-bw', fmtB((m.bytes_per_second_in||0)+(m.bytes_per_second_out||0)));
            setVal('kpi-in', fmtB(m.bytes_per_second_in||0));
            setVal('kpi-out', fmtB(m.bytes_per_second_out||0));
            setVal('kpi-checks', h.healthy_layers||0);

            // Deep
            setVal('v-addr', (s.external_addresses||[]).join(', ')||'-');
            setVal('v-cin', s.inbound_connections||0);
            setVal('v-cout', s.outbound_connections||0);
            setVal('v-err', s.connection_errors||0);

            setVal('v-dht', s.kademlia_routing_table_size||0);
            setVal('v-mdns', s.mdns_discovered_count||0);
            setVal('v-recs', s.dht_records_stored||0);

            setVal('v-nat', (s.nat_status||'?').toUpperCase());
            setVal('v-dcutr', (s.dcutr_success_rate||0).toFixed(0)+'%');
            setVal('v-onion', m.onion_circuits_built||0);

            if(s.peers) renderPeers(s.peers);
            if(d.layers) renderLayers(d.layers);
            if(d.recent_events) renderLog(d.recent_events);
        }}

        async function copy(txt) {{
            await navigator.clipboard.writeText(txt);
            $('toast').classList.add('show');
            setTimeout(() => $('toast').classList.remove('show'), 2000);
        }}

        fetch('/diagnostics').then(r=>r.json()).then(update);
        const sse = new EventSource('/diagnostics/stream');
        sse.onmessage = e => update(JSON.parse(e.data));
    </script>
</body>
</html>
"##,
        peer_id = peer_id
    )
}
