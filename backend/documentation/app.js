// Load report data
const reportDataElement = document.getElementById('report-data');
if (!reportDataElement) {
  console.error('Report data not found in HTML. Ensure the visualization was generated correctly.');
  document.body.innerHTML = '<div style="padding: 20px; color: red;">Error: Report data not found.</div>';
  throw new Error('Report data missing');
}
const reportData = JSON.parse(reportDataElement.textContent);

// Update stats
document.getElementById('stat-total').textContent = reportData.summary.total_dependencies;
document.getElementById('stat-duplicates').textContent = reportData.summary.duplicate_crates;
document.getElementById('stat-vulns').textContent = reportData.diagnostics.vulnerabilities.length;

// Theme toggle
document.getElementById('theme-toggle').addEventListener('click', () => {
  document.body.classList.toggle('light-mode');
});

// Graph setup
const svg = d3.select('#graph');
let width = window.innerWidth;
let height = window.innerHeight - 60;

svg.attr('viewBox', [0, 0, width, height]);

const g = svg.append('g');

// Zoom
const zoom = d3.zoom()
  .scaleExtent([0.1, 4])
  .on('zoom', (event) => {
    g.attr('transform', event.transform);
  });

svg.call(zoom);

document.getElementById('reset-zoom').addEventListener('click', () => {
  svg.transition().duration(750).call(zoom.transform, d3.zoomIdentity);
});

// Build graph data
const nodes = reportData.graph.nodes.map((n) => ({ ...n }));
const links = reportData.graph.edges.map((e) => ({
  source: e.from,
  target: e.to,
  kind: e.kind,
}));

// Create vulnerable set
const vulnerableSet = new Set(
  reportData.diagnostics.vulnerabilities.map((v) => v.package)
);

// Force simulation
const simulation = d3.forceSimulation(nodes)
  .force('link', d3.forceLink(links).id((d) => d.id).distance(100))
  .force('charge', d3.forceManyBody().strength(-300))
  .force('center', d3.forceCenter(width / 2, height / 2))
  .force('collision', d3.forceCollide().radius(30));

// Links
const link = g.append('g')
  .selectAll('line')
  .data(links)
  .join('line')
  .attr('class', (d) => `link ${d.kind}`);

// Nodes
const node = g.append('g')
  .selectAll('g')
  .data(nodes)
  .join('g')
  .attr('class', (d) => {
    if (vulnerableSet.has(d.name)) return 'node vulnerable';
    return `node ${d.kind}`;
  })
  .call(drag(simulation));

node.append('circle').attr('r', (d) => (d.kind === 'root' ? 10 : 6));

node.append('text').text((d) => d.name).attr('x', 12).attr('y', 4);

// Click handler
node.on('click', (event, d) => {
  const panel = document.getElementById('info-panel');
  panel.classList.add('visible');
  document.getElementById('info-name').textContent = d.name;
  document.getElementById('info-version').textContent = d.version;
  document.getElementById('info-type').textContent = d.kind;
  document.getElementById('info-deps').textContent = links.filter((l) => l.source.id === d.id).length;
});

// Search
document.getElementById('search').addEventListener('input', (e) => {
  const query = e.target.value.toLowerCase();
  node.style('opacity', (d) => (!query || d.name.toLowerCase().includes(query) ? 1 : 0.1));
});

// Simulation tick
simulation.on('tick', () => {
  link
    .attr('x1', (d) => d.source.x)
    .attr('y1', (d) => d.source.y)
    .attr('x2', (d) => d.target.x)
    .attr('y2', (d) => d.target.y);

  node.attr('transform', (d) => `translate(${d.x},${d.y})`);
});

// Drag behavior
function drag(simulation) {
  function dragstarted(event) {
    if (!event.active) simulation.alphaTarget(0.3).restart();
    event.subject.fx = event.subject.x;
    event.subject.fy = event.subject.y;
  }

  function dragged(event) {
    event.subject.fx = event.x;
    event.subject.fy = event.y;
  }

  function dragended(event) {
    if (!event.active) simulation.alphaTarget(0);
    event.subject.fx = null;
    event.subject.fy = null;
  }

  return d3.drag().on('start', dragstarted).on('drag', dragged).on('end', dragended);
}

// Resize handling
window.addEventListener('resize', () => {
  width = window.innerWidth;
  height = window.innerHeight - 60;
  svg.attr('viewBox', [0, 0, width, height]);
  simulation.force('center', d3.forceCenter(width / 2, height / 2));
  simulation.alpha(0.3).restart();
});
