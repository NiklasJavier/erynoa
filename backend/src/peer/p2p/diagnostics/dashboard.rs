pub fn generate_dashboard_html(peer_id: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Erynoa Diagnostics Portal</title>
    <style>
        :root {{
            --bg: #fafafa;
            --fg: #09090b;
            --card: #ffffff;
            --card-fg: #09090b;
            --border: #e4e4e7;
            --muted: #f4f4f5;
            --muted-fg: #71717a;
            --primary: #18181b;
            --primary-fg: #fafafa;
            --success: #16a34a; --success-bg: #dcfce7; --success-border: #bbf7d0;
            --warning: #ca8a04; --warning-bg: #fef9c3; --warning-border: #fde68a;
            --error: #dc2626;   --error-bg: #fee2e2;   --error-border: #fecaca;
            --info: #2563eb;    --info-bg: #dbeafe;    --info-border: #bfdbfe;
            --radius: 8px;
            --font-sans: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            --font-mono: "JetBrains Mono", "SF Mono", "Fira Code", monospace;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--bg);
            color: var(--fg);
            font-family: var(--font-sans);
            font-size: 13px;
            line-height: 1.5;
            -webkit-font-smoothing: antialiased;
            padding: 20px;
        }}
        .container {{ max-width: 1800px; margin: 0 auto; }}

        /* Header */
        .header {{
            display: flex; justify-content: space-between; align-items: center;
            margin-bottom: 20px; padding-bottom: 16px; border-bottom: 1px solid var(--border);
        }}
        .logo {{ display: flex; align-items: center; gap: 12px; }}
        .logo-icon {{
            width: 40px; height: 40px; background: linear-gradient(135deg, #18181b 0%, #3f3f46 100%);
            border-radius: 10px; display: grid; place-items: center;
            color: white; font-weight: 800; font-size: 20px;
        }}
        .logo-text h1 {{ font-size: 18px; font-weight: 700; letter-spacing: -0.02em; }}
        .logo-text .peer-id {{
            font-family: var(--font-mono); font-size: 11px; color: var(--muted-fg);
            cursor: pointer; padding: 2px 6px; border-radius: 4px; background: var(--muted);
        }}
        .logo-text .peer-id:hover {{ background: var(--border); }}
        .status-group {{ text-align: right; }}
        .status-badge {{
            display: inline-flex; align-items: center; gap: 6px;
            padding: 6px 12px; border-radius: 20px; font-size: 11px; font-weight: 600;
        }}
        .status-ok {{ background: var(--success-bg); color: #14532d; border: 1px solid var(--success-border); }}
        .status-warn {{ background: var(--warning-bg); color: #713f12; border: 1px solid var(--warning-border); }}
        .status-err {{ background: var(--error-bg); color: #7f1d1d; border: 1px solid var(--error-border); }}
        .uptime {{ font-family: var(--font-mono); font-size: 12px; color: var(--muted-fg); margin-top: 4px; }}
        .pulse {{ width: 8px; height: 8px; background: var(--success); border-radius: 50%; animation: pulse 2s infinite; }}
        @keyframes pulse {{ 0%,100% {{ opacity: 1; }} 50% {{ opacity: 0.5; }} }}

        /* KPI Grid */
        .kpi-grid {{
            display: grid; grid-template-columns: repeat(6, 1fr); gap: 12px; margin-bottom: 20px;
        }}
        @media (max-width: 1400px) {{ .kpi-grid {{ grid-template-columns: repeat(3, 1fr); }} }}
        @media (max-width: 800px) {{ .kpi-grid {{ grid-template-columns: repeat(2, 1fr); }} }}
        .kpi {{
            background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
            padding: 14px 16px; display: flex; flex-direction: column;
        }}
        .kpi-label {{ font-size: 10px; font-weight: 600; color: var(--muted-fg); text-transform: uppercase; letter-spacing: 0.05em; }}
        .kpi-val {{ font-size: 28px; font-weight: 700; letter-spacing: -0.03em; margin: 4px 0 2px; display: flex; align-items: baseline; gap: 4px; }}
        .kpi-val .unit {{ font-size: 14px; font-weight: 500; color: var(--muted-fg); }}
        .kpi-sub {{ font-size: 11px; color: var(--muted-fg); display: flex; align-items: center; gap: 4px; }}
        .kpi-sub .dot {{ width: 6px; height: 6px; border-radius: 50%; }}
        .kpi-sub .green {{ background: var(--success); }}
        .kpi-sub .yellow {{ background: var(--warning); }}
        .kpi-sub .red {{ background: var(--error); }}

        /* Main Grid */
        .main-grid {{
            display: grid; grid-template-columns: 340px 1fr 400px; gap: 16px;
            align-items: start;
        }}
        @media (max-width: 1400px) {{ .main-grid {{ grid-template-columns: 300px 1fr; }} }}
        @media (max-width: 1000px) {{ .main-grid {{ grid-template-columns: 1fr; }} }}
        .col {{ display: flex; flex-direction: column; gap: 16px; }}

        /* Cards */
        .card {{
            background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
            overflow: hidden;
        }}
        .card-head {{
            padding: 12px 16px; border-bottom: 1px solid var(--border);
            display: flex; justify-content: space-between; align-items: center;
            background: linear-gradient(to bottom, #fafafa, #f4f4f5);
        }}
        .card-title {{ font-size: 12px; font-weight: 600; display: flex; align-items: center; gap: 8px; }}
        .card-title .icon {{ font-size: 14px; }}
        .card-body {{ padding: 0; }}

        /* Badges */
        .badge {{
            display: inline-flex; align-items: center; padding: 2px 8px;
            border-radius: 4px; font-size: 10px; font-weight: 600; white-space: nowrap;
        }}
        .b-outline {{ background: var(--muted); color: var(--fg); }}
        .b-green {{ background: var(--success-bg); color: #14532d; }}
        .b-yellow {{ background: var(--warning-bg); color: #713f12; }}
        .b-red {{ background: var(--error-bg); color: #7f1d1d; }}
        .b-blue {{ background: var(--info-bg); color: #1e40af; }}
        .b-dark {{ background: var(--primary); color: var(--primary-fg); }}

        /* Data Rows */
        .row {{
            display: flex; justify-content: space-between; align-items: center;
            padding: 10px 16px; border-bottom: 1px solid var(--border);
            font-size: 12px;
        }}
        .row:last-child {{ border-bottom: none; }}
        .row-label {{ color: var(--muted-fg); display: flex; align-items: center; gap: 6px; }}
        .row-label .icon {{ font-size: 12px; opacity: 0.7; }}
        .row-val {{ font-family: var(--font-mono); font-weight: 500; }}

        /* Section Header */
        .section {{ padding: 8px 16px; background: var(--muted); font-size: 10px; font-weight: 600; color: var(--muted-fg); text-transform: uppercase; letter-spacing: 0.05em; }}

        /* Layer Stack */
        .layer {{
            border: 1px solid var(--border); border-radius: 6px; margin: 10px 12px; overflow: hidden;
        }}
        .layer-head {{
            padding: 8px 12px; background: linear-gradient(to right, #fafafa, #f4f4f5);
            display: flex; justify-content: space-between; align-items: center;
            font-size: 11px; font-weight: 600; border-bottom: 1px solid var(--border);
        }}
        .layer-body {{ padding: 0; }}
        .check {{
            padding: 6px 12px; display: flex; gap: 8px; font-size: 11px;
            border-bottom: 1px solid var(--border); align-items: flex-start;
        }}
        .check:last-child {{ border-bottom: none; }}
        .check-dot {{ width: 6px; height: 6px; border-radius: 50%; margin-top: 5px; flex-shrink: 0; }}
        .check-name {{ font-weight: 600; color: var(--fg); }}
        .check-msg {{ color: var(--muted-fg); }}
        .check-val {{ font-family: var(--font-mono); font-weight: 500; color: var(--fg); }}

        /* Charts */
        .chart-wrap {{ position: relative; height: 80px; background: #fafafa; }}
        canvas {{ width: 100%; height: 100%; }}
        .chart-legend {{
            position: absolute; top: 8px; right: 12px; display: flex; gap: 12px; font-size: 10px;
        }}
        .chart-legend span {{ display: flex; align-items: center; gap: 4px; }}
        .chart-legend .dot {{ width: 8px; height: 8px; border-radius: 2px; }}
        .mini-stats {{
            display: grid; grid-template-columns: repeat(2, 1fr); gap: 1px;
            background: var(--border);
        }}
        .mini-stat {{
            background: var(--card); padding: 12px; text-align: center;
        }}
        .mini-stat-val {{ font-size: 20px; font-weight: 700; font-family: var(--font-mono); }}
        .mini-stat-label {{ font-size: 9px; color: var(--muted-fg); text-transform: uppercase; margin-top: 2px; }}

        /* Peer Table */
        .peer-table {{ width: 100%; border-collapse: collapse; font-size: 11px; }}
        .peer-table th {{
            text-align: left; padding: 10px 12px; background: var(--muted);
            font-weight: 600; color: var(--muted-fg); font-size: 10px; text-transform: uppercase;
            letter-spacing: 0.03em; border-bottom: 1px solid var(--border);
        }}
        .peer-table td {{
            padding: 8px 12px; border-bottom: 1px solid var(--border);
            font-family: var(--font-mono);
        }}
        .peer-table tr:last-child td {{ border-bottom: none; }}
        .peer-table .peer-id {{ font-weight: 500; cursor: pointer; }}
        .peer-table .peer-id:hover {{ color: var(--info); }}
        .peer-addr {{ font-size: 10px; color: var(--muted-fg); margin-top: 2px; }}
        .peer-flags {{ display: flex; gap: 4px; justify-content: flex-end; }}
        .peer-flag {{
            width: 18px; height: 18px; border-radius: 4px; display: grid; place-items: center;
            font-size: 9px; font-weight: 700;
        }}
        .flag-r {{ background: #dbeafe; color: #1e40af; }}
        .flag-g {{ background: #dcfce7; color: #14532d; }}
        .flag-k {{ background: #fef9c3; color: #713f12; }}

        /* Event Log */
        .log-item {{
            padding: 10px 12px; border-bottom: 1px solid var(--border);
            display: flex; gap: 10px; font-size: 11px;
        }}
        .log-icon {{ font-size: 12px; flex-shrink: 0; }}
        .log-msg {{ color: var(--fg); line-height: 1.4; }}
        .log-meta {{ font-family: var(--font-mono); font-size: 9px; color: var(--muted-fg); margin-top: 2px; }}

        /* Animations */
        .flash {{ animation: flash-anim 0.4s ease-out; }}
        @keyframes flash-anim {{ 0% {{ background: rgba(37, 99, 235, 0.1); }} 100% {{ background: transparent; }} }}
        @keyframes slide-in {{ from {{ opacity: 0; transform: translateY(-4px); }} to {{ opacity: 1; transform: translateY(0); }} }}
        .log-item {{ animation: slide-in 0.2s ease-out; }}

        /* Toast */
        .toast {{
            position: fixed; bottom: 24px; right: 24px;
            background: var(--primary); color: var(--primary-fg);
            padding: 10px 16px; border-radius: 8px; font-size: 12px; font-weight: 500;
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
            transform: translateY(20px); opacity: 0; pointer-events: none;
            transition: all 0.2s ease; z-index: 1000;
        }}
        .toast.show {{ transform: translateY(0); opacity: 1; }}

        ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
        ::-webkit-scrollbar-track {{ background: transparent; }}
        ::-webkit-scrollbar-thumb {{ background: var(--border); border-radius: 3px; }}
        ::-webkit-scrollbar-thumb:hover {{ background: var(--muted-fg); }}
    </style>
</head>
<body>
    <div class="container">
        <!-- Header -->
        <div class="header">
            <div class="logo">
                <div class="logo-icon">E</div>
                <div class="logo-text">
                    <h1>Erynoa Diagnostics Portal</h1>
                    <span class="peer-id" onclick="copy('{peer_id}')" title="Click to copy">{peer_id}</span>
                </div>
            </div>
            <div class="status-group">
                <div class="status-badge status-ok" id="status-badge">
                    <span class="pulse"></span>
                    <span id="status-text">HEALTHY</span>
                </div>
                <div class="uptime">Uptime: <span id="uptime">00:00:00</span></div>
            </div>
        </div>

        <!-- KPI Strip -->
        <div class="kpi-grid">
            <div class="kpi">
                <span class="kpi-label">Connected Peers</span>
                <span class="kpi-val"><span id="kpi-peers">0</span></span>
                <span class="kpi-sub"><span class="dot green"></span> <span id="kpi-in-conn">0</span> in / <span id="kpi-out-conn">0</span> out</span>
            </div>
            <div class="kpi">
                <span class="kpi-label">Gossip Mesh</span>
                <span class="kpi-val"><span id="kpi-mesh">0</span><span class="unit">peers</span></span>
                <span class="kpi-sub">📨 <span id="kpi-msg-rx">0</span> rx / <span id="kpi-msg-tx">0</span> tx</span>
            </div>
            <div class="kpi">
                <span class="kpi-label">Avg Latency</span>
                <span class="kpi-val"><span id="kpi-lat">0</span><span class="unit">ms</span></span>
                <span class="kpi-sub">min <span id="kpi-lat-min">0</span> / max <span id="kpi-lat-max">0</span></span>
            </div>
            <div class="kpi">
                <span class="kpi-label">Kademlia DHT</span>
                <span class="kpi-val"><span id="kpi-kad">0</span><span class="unit">peers</span></span>
                <span class="kpi-sub"><span class="dot" id="kpi-kad-dot"></span> <span id="kpi-kad-status">Bootstrapping</span></span>
            </div>
            <div class="kpi">
                <span class="kpi-label">Relay Circuits</span>
                <span class="kpi-val"><span id="kpi-relay">0</span><span class="unit">serving</span></span>
                <span class="kpi-sub"><span class="dot" id="kpi-relay-dot"></span> <span id="kpi-relay-status">-</span></span>
            </div>
            <div class="kpi">
                <span class="kpi-label">Health Score</span>
                <span class="kpi-val" id="kpi-health" style="color:var(--success)">100<span class="unit">%</span></span>
                <span class="kpi-sub"><span id="kpi-checks">0</span>/<span id="kpi-total">0</span> checks passed</span>
            </div>
        </div>

        <!-- System KPIs -->
        <div style="margin: 16px 0 8px; font-size: 12px; font-weight: 600; color: var(--muted-fg); text-transform: uppercase; letter-spacing: 0.05em;">
            System Modules (Core, ECLVM, Local, Protection)
        </div>
        <div class="kpi-grid">
            <div class="kpi">
                <span class="kpi-label">Trust Engine (Κ2-Κ5)</span>
                <span class="kpi-val"><span id="sys-trust-entities">0</span><span class="unit">entities</span></span>
                <span class="kpi-sub">avg trust: <span id="sys-trust-avg">0.000</span></span>
            </div>
            <div class="kpi">
                <span class="kpi-label">Event Engine (Κ9-Κ12)</span>
                <span class="kpi-val"><span id="sys-events-total">0</span><span class="unit">events</span></span>
                <span class="kpi-sub"><span id="sys-events-finalized">0</span> finalized</span>
            </div>
            <div class="kpi">
                <span class="kpi-label">World Formula 𝔼</span>
                <span class="kpi-val"><span id="sys-wf-e">0.0000</span></span>
                <span class="kpi-sub"><span id="sys-wf-contrib">0</span> contributors</span>
            </div>
            <div class="kpi">
                <span class="kpi-label">Consensus (Κ18)</span>
                <span class="kpi-val">Epoch <span id="sys-consensus-epoch">0</span></span>
                <span class="kpi-sub"><span id="sys-consensus-validators">0</span> validators</span>
            </div>
            <div class="kpi">
                <span class="kpi-label">ECLVM Runtime</span>
                <span class="kpi-val"><span id="sys-eclvm-executed">0</span><span class="unit">programs</span></span>
                <span class="kpi-sub"><span id="sys-eclvm-gas">0</span> gas consumed</span>
            </div>
            <div class="kpi">
                <span class="kpi-label">System Health</span>
                <span class="kpi-val" id="sys-health" style="color:var(--success)">100%</span>
                <span class="kpi-sub"><span id="sys-anomalies">0</span> anomalies, H=<span id="sys-diversity">0.00</span></span>
            </div>
        </div>

        <!-- Main Dashboard -->
        <div class="main-grid">
            <!-- Left Column: Layer Stack -->
            <div class="col">
                <div class="card" style="max-height: 820px;">
                    <div class="card-head">
                        <span class="card-title"><span class="icon">🏗️</span> P2P Infrastructure Layers</span>
                        <span class="badge b-outline" id="layer-count">8 Layers</span>
                    </div>
                    <div class="card-body" id="layers" style="overflow-y: auto; max-height: 760px;"></div>
                </div>
            </div>

            <!-- Center Column: Live Data -->
            <div class="col">
                <!-- Traffic Chart -->
                <div class="card">
                    <div class="card-head">
                        <span class="card-title"><span class="icon">📊</span> Network Traffic (60s)</span>
                        <span class="pulse"></span>
                    </div>
                    <div class="chart-wrap">
                        <canvas id="chart"></canvas>
                        <div class="chart-legend">
                            <span><span class="dot" style="background:#2563eb"></span> Inbound</span>
                            <span><span class="dot" style="background:#16a34a"></span> Outbound</span>
                        </div>
                    </div>
                    <div class="mini-stats">
                        <div class="mini-stat">
                            <div class="mini-stat-val" id="bw-in">0 B/s</div>
                            <div class="mini-stat-label">Inbound</div>
                        </div>
                        <div class="mini-stat">
                            <div class="mini-stat-val" id="bw-out">0 B/s</div>
                            <div class="mini-stat-label">Outbound</div>
                        </div>
                    </div>
                </div>

                <!-- Network Transport -->
                <div class="card">
                    <div class="card-head">
                        <span class="card-title"><span class="icon">🔌</span> Network Transport</span>
                    </div>
                    <div class="card-body">
                        <div class="row">
                            <span class="row-label"><span class="icon">🌐</span> External Addresses</span>
                            <span class="row-val" id="v-addr" style="font-size:10px; max-width:200px; overflow:hidden; text-overflow:ellipsis;">-</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">↔️</span> Connections</span>
                            <span class="row-val"><span id="v-cin">0</span> ↓ / <span id="v-cout">0</span> ↑</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">❌</span> Connection Errors</span>
                            <span class="row-val" id="v-err" style="color:var(--error)">0</span>
                        </div>
                    </div>
                </div>

                <!-- Discovery & DHT -->
                <div class="card">
                    <div class="card-head">
                        <span class="card-title"><span class="icon">🔍</span> Discovery & DHT</span>
                    </div>
                    <div class="card-body">
                        <div class="row">
                            <span class="row-label"><span class="icon">📡</span> Kademlia Routing Table</span>
                            <span class="row-val"><span id="v-kad">0</span> peers</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">🏠</span> mDNS Local Discovery</span>
                            <span class="row-val"><span id="v-mdns">0</span> discovered</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">💾</span> DHT Records Stored</span>
                            <span class="row-val" id="v-recs">0</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">🚀</span> Bootstrap Status</span>
                            <span class="badge" id="v-bootstrap">-</span>
                        </div>
                    </div>
                </div>

                <!-- NAT Traversal -->
                <div class="card">
                    <div class="card-head">
                        <span class="card-title"><span class="icon">🛡️</span> NAT Traversal</span>
                    </div>
                    <div class="card-body">
                        <div class="row">
                            <span class="row-label"><span class="icon">🌍</span> NAT Status (AutoNAT)</span>
                            <span class="badge" id="v-nat">UNKNOWN</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">🔄</span> UPnP Port Mapping</span>
                            <span class="badge" id="v-upnp">-</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">🎯</span> DCUTR Holepunching</span>
                            <span class="row-val"><span id="v-dcutr-succ">0</span>✓ / <span id="v-dcutr-fail">0</span>✗ (<span id="v-dcutr">0</span>%)</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">📡</span> Relay Reservation</span>
                            <span class="badge" id="v-relay-res">-</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">🔗</span> Relay Circuits Serving</span>
                            <span class="row-val" id="v-relay-circ">0</span>
                        </div>
                    </div>
                </div>

                <!-- Gossipsub -->
                <div class="card">
                    <div class="card-head">
                        <span class="card-title"><span class="icon">📣</span> Gossipsub PubSub</span>
                    </div>
                    <div class="card-body">
                        <div class="row">
                            <span class="row-label"><span class="icon">🕸️</span> Mesh Size</span>
                            <span class="row-val"><span id="v-mesh">0</span> peers</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">📋</span> Topics Subscribed</span>
                            <span class="row-val" id="v-topics">0</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">📥</span> Messages Received</span>
                            <span class="row-val" id="v-msg-rx">0</span>
                        </div>
                        <div class="row">
                            <span class="row-label"><span class="icon">📤</span> Messages Sent</span>
                            <span class="row-val" id="v-msg-tx">0</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Right Column: Peers & Events -->
            <div class="col">
                <!-- Connected Peers -->
                <div class="card" style="max-height: 420px;">
                    <div class="card-head">
                        <span class="card-title"><span class="icon">👥</span> Connected Peers</span>
                        <span class="badge b-dark" id="badge-peers">0</span>
                    </div>
                    <div class="card-body" style="overflow-y: auto; max-height: 360px;">
                        <table class="peer-table">
                            <thead>
                                <tr>
                                    <th>Peer ID</th>
                                    <th style="text-align:right">RTT</th>
                                    <th style="text-align:right">Flags</th>
                                </tr>
                            </thead>
                            <tbody id="peer-rows"></tbody>
                        </table>
                    </div>
                </div>

                <!-- Event Log -->
                <div class="card" style="max-height: 380px;">
                    <div class="card-head">
                        <span class="card-title"><span class="icon">📜</span> Event Log</span>
                        <span class="badge b-outline" id="event-count">0 events</span>
                    </div>
                    <div class="card-body" id="logs" style="overflow-y: auto; max-height: 320px; background: #fafafa;"></div>
                </div>
            </div>
        </div>

        <!-- System Layers Section (Below Main Grid) -->
        <div style="margin-top: 20px;">
            <div class="card">
                <div class="card-head">
                    <span class="card-title"><span class="icon">⚙️</span> System Module Layers (Core, ECLVM, Local, Protection)</span>
                    <span class="badge b-outline" id="system-layer-count">13 Layers</span>
                </div>
                <div class="card-body" id="system-layers" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 8px; padding: 12px; max-height: 600px; overflow-y: auto;">
                    <div style="text-align: center; color: var(--muted-fg); padding: 40px;">
                        Waiting for system metrics...
                    </div>
                </div>
            </div>
        </div>
    </div>

    <div id="toast" class="toast">Copied to clipboard!</div>

    <script>
        const $ = id => document.getElementById(id);
        const fmtB = b => {{
            if (!+b) return '0 B/s';
            const k = 1024, sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
            const i = Math.floor(Math.log(b) / Math.log(k));
            return parseFloat((b / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
        }};
        const fmtT = s => {{
            const h = Math.floor(s / 3600);
            const m = Math.floor((s % 3600) / 60);
            const sec = s % 60;
            return `${{String(h).padStart(2,'0')}}:${{String(m).padStart(2,'0')}}:${{String(sec).padStart(2,'0')}}`;
        }};
        const fmtNum = n => n >= 1000000 ? (n/1000000).toFixed(1)+'M' : n >= 1000 ? (n/1000).toFixed(1)+'K' : n;

        // Chart
        const canvas = $('chart');
        const ctx = canvas.getContext('2d');
        const chartData = {{ in: new Array(60).fill(0), out: new Array(60).fill(0) }};

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
        resizeChart();

        function drawChart() {{
            const w = canvas.width / (window.devicePixelRatio || 1);
            const h = canvas.height / (window.devicePixelRatio || 1);
            ctx.clearRect(0, 0, w, h);

            ctx.strokeStyle = '#e4e4e7';
            ctx.lineWidth = 0.5;
            for (let i = 0; i < 4; i++) {{
                const y = (h / 4) * i;
                ctx.beginPath();
                ctx.moveTo(0, y);
                ctx.lineTo(w, y);
                ctx.stroke();
            }}

            const max = Math.max(1024, ...chartData.in, ...chartData.out) * 1.2;
            const step = w / 59;

            const drawLine = (arr, color, fillColor) => {{
                ctx.beginPath();
                ctx.moveTo(0, h - (arr[0] / max * h));
                arr.forEach((v, i) => ctx.lineTo(i * step, h - (v / max * h)));
                ctx.strokeStyle = color;
                ctx.lineWidth = 2;
                ctx.stroke();

                ctx.lineTo(w, h);
                ctx.lineTo(0, h);
                ctx.closePath();
                ctx.fillStyle = fillColor;
                ctx.fill();
            }};

            drawLine(chartData.in, '#2563eb', 'rgba(37, 99, 235, 0.08)');
            drawLine(chartData.out, '#16a34a', 'rgba(22, 163, 74, 0.08)');
        }}

        function setVal(id, txt) {{
            const el = $(id);
            if (!el) return;
            const s = String(txt);
            if (el.innerText !== s) {{
                el.innerText = s;
                el.classList.remove('flash');
                void el.offsetWidth;
                el.classList.add('flash');
            }}
        }}

        function renderLayers(layers) {{
            if (!layers) return;
            let html = '';
            layers.forEach(l => {{
                const st = l.overall_status;
                const col = st === 'healthy' ? 'var(--success)' : st === 'degraded' ? 'var(--warning)' : 'var(--error)';
                const badge = st === 'healthy' ? 'b-green' : st === 'degraded' ? 'b-yellow' : 'b-red';

                html += `<div class="layer">
                    <div class="layer-head">
                        <span>${{l.layer_name}}</span>
                        <span class="badge ${{badge}}">${{st.toUpperCase()}}</span>
                    </div>
                    <div class="layer-body">`;

                l.checks.forEach(c => {{
                    const dot = c.status === 'healthy' ? 'var(--success)' : c.status === 'degraded' ? 'var(--warning)' : 'var(--error)';
                    const val = c.metric_value != null ? `<span class="check-val">${{c.metric_value % 1 !== 0 ? c.metric_value.toFixed(1) : c.metric_value}}${{c.metric_unit || ''}}</span> ` : '';
                    html += `<div class="check">
                        <div class="check-dot" style="background:${{dot}}"></div>
                        <div style="flex:1">
                            <span class="check-name">${{c.name}}</span>
                            <div class="check-msg">${{val}}${{c.message}}</div>
                        </div>
                    </div>`;
                }});

                html += `</div></div>`;
            }});
            $('layers').innerHTML = html;
            $('layer-count').innerText = layers.length + ' Layers';
        }}

        function renderPeers(peers) {{
            const el = $('peer-rows');
            $('badge-peers').innerText = peers.length;

            if (!peers.length) {{
                el.innerHTML = '<tr><td colspan="3" style="text-align:center; padding:30px; color:var(--muted-fg)">No connected peers</td></tr>';
                return;
            }}

            el.innerHTML = peers.map(p => {{
                const rtt = p.ping_rtt_ms ?? 0;
                const rttColor = rtt < 50 ? 'var(--success)' : rtt < 150 ? 'var(--warning)' : 'var(--error)';

                let flags = '';
                if (p.is_relayed) flags += '<span class="peer-flag flag-r" title="Relayed">R</span>';
                if (p.in_gossip_mesh) flags += '<span class="peer-flag flag-g" title="In Gossip Mesh">G</span>';
                if (p.in_kademlia) flags += '<span class="peer-flag flag-k" title="In Kademlia">K</span>';

                return `<tr>
                    <td>
                        <div class="peer-id" onclick="copy('${{p.peer_id}}')">${{p.peer_id.substring(0, 12)}}...</div>
                        <div class="peer-addr">${{p.address || 'No address'}}</div>
                    </td>
                    <td style="text-align:right; color:${{rttColor}}">${{rtt}}ms</td>
                    <td><div class="peer-flags">${{flags || '-'}}</div></td>
                </tr>`;
            }}).join('');
        }}

        let eventCount = 0;
        const seenEvents = new Set();
        function renderEvents(events) {{
            if (!events) return;
            const container = $('logs');

            events.forEach(e => {{
                if (seenEvents.has(e.id)) return;
                seenEvents.add(e.id);
                eventCount++;

                let icon = 'ℹ️';
                const t = (e.event_type || '').toLowerCase();
                if (e.severity === 'error' || t.includes('fail') || t.includes('error')) icon = '🔥';
                else if (e.severity === 'warning') icon = '⚠️';
                else if (t.includes('success') || t.includes('complete') || t.includes('established')) icon = '✅';
                else if (t.includes('gossip') || t.includes('message')) icon = '📨';
                else if (t.includes('connect')) icon = '🔗';
                else if (t.includes('disconnect')) icon = '👋';
                else if (t.includes('discovery') || t.includes('mdns')) icon = '🔍';

                const html = `<div class="log-item">
                    <span class="log-icon">${{icon}}</span>
                    <div style="flex:1">
                        <div class="log-msg">${{e.message}}</div>
                        <div class="log-meta">${{e.timestamp?.split('T')[1]?.split('.')[0] || ''}} • ${{e.event_type}}</div>
                    </div>
                </div>`;

                container.insertAdjacentHTML('afterbegin', html);
                if (container.children.length > 100) container.lastElementChild.remove();
            }});

            $('event-count').innerText = eventCount + ' events';
        }}

        function update(d) {{
            const m = d.metrics || {{}};
            const s = d.swarm || {{}};
            const h = d.health || {{}};
            const summary = d.summary || {{}};

            // Chart data
            chartData.in.shift(); chartData.in.push(m.bytes_per_second_in || 0);
            chartData.out.shift(); chartData.out.push(m.bytes_per_second_out || 0);
            drawChart();

            // Bandwidth
            $('bw-in').innerText = fmtB(m.bytes_per_second_in || 0);
            $('bw-out').innerText = fmtB(m.bytes_per_second_out || 0);

            // Status
            const status = h.status || (summary.health_percentage >= 90 ? 'HEALTHY' : summary.health_percentage >= 50 ? 'DEGRADED' : 'CRITICAL');
            const statusBadge = $('status-badge');
            statusBadge.className = 'status-badge ' + (status === 'HEALTHY' ? 'status-ok' : status === 'DEGRADED' ? 'status-warn' : 'status-err');
            $('status-text').innerText = status;
            setVal('uptime', fmtT(s.uptime_secs || 0));

            // KPIs
            setVal('kpi-peers', s.connected_peers_count ?? 0);
            setVal('kpi-in-conn', s.inbound_connections ?? 0);
            setVal('kpi-out-conn', s.outbound_connections ?? 0);

            setVal('kpi-mesh', s.gossip_mesh_size ?? 0);
            setVal('kpi-msg-rx', fmtNum(s.gossip_messages_received ?? 0));
            setVal('kpi-msg-tx', fmtNum(s.gossip_messages_sent ?? 0));

            setVal('kpi-lat', (s.avg_ping_ms || 0).toFixed(0));
            setVal('kpi-lat-min', (s.min_ping_ms || 0).toFixed(0));
            setVal('kpi-lat-max', (s.max_ping_ms || 0).toFixed(0));

            setVal('kpi-kad', s.kademlia_routing_table_size ?? 0);
            const kadDot = $('kpi-kad-dot');
            kadDot.className = 'dot ' + (s.kademlia_bootstrap_complete ? 'green' : 'yellow');
            $('kpi-kad-status').innerText = s.kademlia_bootstrap_complete ? 'Ready' : 'Bootstrapping';

            setVal('kpi-relay', s.relay_circuits_serving ?? 0);
            const relayDot = $('kpi-relay-dot');
            relayDot.className = 'dot ' + (s.has_relay_reservation ? 'green' : 'yellow');
            $('kpi-relay-status').innerText = s.has_relay_reservation ? 'Reserved' : 'No reservation';

            // Health
            const healthPct = summary.health_percentage ?? 100;
            $('kpi-health').style.color = healthPct >= 90 ? 'var(--success)' : healthPct >= 50 ? 'var(--warning)' : 'var(--error)';
            setVal('kpi-health', healthPct.toFixed(0) + '%');
            setVal('kpi-checks', summary.healthy_count ?? 0);
            setVal('kpi-total', summary.total_checks ?? 0);

            // Network Transport
            const addrs = s.external_addresses || [];
            $('v-addr').innerText = addrs.length ? addrs.join(', ') : 'None detected';
            $('v-addr').title = addrs.join('\n');
            setVal('v-cin', s.inbound_connections ?? 0);
            setVal('v-cout', s.outbound_connections ?? 0);
            setVal('v-err', s.connection_errors ?? 0);

            // Discovery
            setVal('v-kad', s.kademlia_routing_table_size ?? 0);
            setVal('v-mdns', s.mdns_discovered_count ?? 0);
            setVal('v-recs', s.dht_records_stored ?? 0);
            const bootBadge = $('v-bootstrap');
            bootBadge.className = 'badge ' + (s.kademlia_bootstrap_complete ? 'b-green' : 'b-yellow');
            bootBadge.innerText = s.kademlia_bootstrap_complete ? 'COMPLETE' : 'IN PROGRESS';

            // NAT Traversal
            const natBadge = $('v-nat');
            const natStatus = (s.nat_status || 'unknown').toUpperCase();
            natBadge.className = 'badge ' + (natStatus === 'PUBLIC' ? 'b-green' : natStatus === 'PRIVATE' ? 'b-yellow' : 'b-outline');
            natBadge.innerText = natStatus;

            const upnpBadge = $('v-upnp');
            upnpBadge.className = 'badge ' + (s.upnp_available ? 'b-green' : 'b-outline');
            upnpBadge.innerText = s.upnp_available ? 'ACTIVE' : 'UNAVAILABLE';

            setVal('v-dcutr-succ', s.dcutr_successes ?? 0);
            setVal('v-dcutr-fail', s.dcutr_failures ?? 0);
            setVal('v-dcutr', (s.dcutr_success_rate ?? 0).toFixed(0));

            const relayResBadge = $('v-relay-res');
            relayResBadge.className = 'badge ' + (s.has_relay_reservation ? 'b-green' : 'b-outline');
            relayResBadge.innerText = s.has_relay_reservation ? 'ACTIVE' : 'NONE';
            setVal('v-relay-circ', s.relay_circuits_serving ?? 0);

            // Gossipsub
            setVal('v-mesh', s.gossip_mesh_size ?? 0);
            setVal('v-topics', s.gossip_topics_subscribed ?? 0);
            setVal('v-msg-rx', fmtNum(s.gossip_messages_received ?? 0));
            setVal('v-msg-tx', fmtNum(s.gossip_messages_sent ?? 0));

            // Peers & Layers
            if (s.peers) renderPeers(s.peers);
            if (d.layers) renderLayers(d.layers);
            if (d.recent_events) renderEvents(d.recent_events);

            // System Layers (Core, ECLVM, Local, Protection)
            if (d.system_layers) renderSystemLayers(d.system_layers);

            // System Metrics KPIs
            if (d.system) updateSystemKPIs(d.system);
        }}

        // Render System-Layers in der zweiten Spalte der Layer-Card
        function renderSystemLayers(layers) {{
            if (!layers) return;
            let html = '';
            layers.forEach(l => {{
                const st = l.overall_status;
                const col = st === 'healthy' ? 'var(--success)' : st === 'degraded' ? 'var(--warning)' : 'var(--error)';
                const badge = st === 'healthy' ? 'b-green' : st === 'degraded' ? 'b-yellow' : 'b-red';

                html += `<div class="layer">
                    <div class="layer-head">
                        <span>${{l.layer_name}}</span>
                        <span class="badge ${{badge}}">${{st.toUpperCase()}}</span>
                    </div>
                    <div class="layer-body">`;

                l.checks.forEach(c => {{
                    const dot = c.status === 'healthy' ? 'var(--success)' : c.status === 'degraded' ? 'var(--warning)' : 'var(--error)';
                    const val = c.metric_value != null ? `<span class="check-val">${{c.metric_value % 1 !== 0 ? c.metric_value.toFixed(1) : c.metric_value}}${{c.metric_unit || ''}}</span> ` : '';
                    html += `<div class="check">
                        <div class="check-dot" style="background:${{dot}}"></div>
                        <div style="flex:1">
                            <span class="check-name">${{c.name}}</span>
                            <div class="check-msg">${{val}}${{c.message}}</div>
                        </div>
                    </div>`;
                }});

                html += `</div></div>`;
            }});
            const el = $('system-layers');
            if (el) {{
                el.innerHTML = html;
                $('system-layer-count').innerText = layers.length + ' Layers';
            }}
        }}

        // Update System KPIs
        function updateSystemKPIs(sys) {{
            if (!sys) return;

            // Trust Engine
            setVal('sys-trust-entities', sys.trust?.entities_count ?? 0);
            setVal('sys-trust-avg', (sys.trust?.avg_trust_value ?? 0).toFixed(3));

            // Event Engine
            setVal('sys-events-total', fmtNum(sys.events?.events_total ?? 0));
            setVal('sys-events-finalized', fmtNum(sys.events?.finalized_events ?? 0));

            // World Formula
            setVal('sys-wf-e', (sys.world_formula?.current_e_value ?? 0).toFixed(4));
            setVal('sys-wf-contrib', sys.world_formula?.contributors_count ?? 0);

            // Consensus
            setVal('sys-consensus-epoch', sys.consensus?.current_epoch ?? 0);
            setVal('sys-consensus-validators', sys.consensus?.validators_count ?? 0);

            // ECLVM
            setVal('sys-eclvm-executed', fmtNum(sys.eclvm?.programs_executed ?? 0));
            setVal('sys-eclvm-gas', fmtNum(sys.eclvm?.total_gas_consumed ?? 0));

            // Mana
            setVal('sys-mana-accounts', sys.mana?.accounts_count ?? 0);
            setVal('sys-mana-consumed', fmtNum(sys.mana?.total_mana_consumed ?? 0));

            // Storage
            setVal('sys-storage-events', fmtNum(sys.storage?.event_store_events ?? 0));
            setVal('sys-storage-identities', sys.storage?.identity_store_entries ?? 0);

            // Protection
            setVal('sys-anomalies', sys.anomaly?.anomalies_detected ?? 0);
            setVal('sys-diversity', (sys.diversity?.min_entropy_current ?? 0).toFixed(2));

            // Health score
            const healthPct = sys.health_score ?? 100;
            const healthEl = $('sys-health');
            if (healthEl) {{
                healthEl.style.color = healthPct >= 90 ? 'var(--success)' : healthPct >= 50 ? 'var(--warning)' : 'var(--error)';
                healthEl.innerText = healthPct.toFixed(0) + '%';
            }}
        }}

        async function copy(txt) {{
            await navigator.clipboard.writeText(txt);
            $('toast').classList.add('show');
            setTimeout(() => $('toast').classList.remove('show'), 2000);
        }}

        fetch('/diagnostics').then(r => r.json()).then(update);

        const sse = new EventSource('/diagnostics/stream');
        sse.onmessage = e => update(JSON.parse(e.data));
        sse.onerror = () => {{
            $('status-text').innerText = 'RECONNECTING...';
            $('status-badge').className = 'status-badge status-warn';
        }};
    </script>
</body>
</html>
"##,
        peer_id = peer_id
    )
}
