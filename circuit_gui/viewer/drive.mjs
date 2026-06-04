// Headless interaction driver for the circuit viewer (no external deps).
//
// Drives real interactions via the Chrome DevTools Protocol: loads a URL, then
// for each step evaluates a JS expression in the page (e.g. expand a group) and
// writes a screenshot. Lets the review agent exercise expand/collapse, circuit
// selection, mode toggles, zoom, etc. — things a static --screenshot can't do.
//
// Usage:
//   node drive.mjs <url> <step1.png> '<step1-js>' [<step2.png> '<step2-js>'] ...
// The JS runs in the page's global scope, where `cy`, `ec`, `setLevel`,
// `fitView`, and the DOM are all reachable. Use "" for a plain screenshot.
//
// Handy snippets to pass as a step's JS:
//   selectCircuit(2)                         // pick circuit by index (fires load)
//   expandByLabel('blake')                   // double-click every collapsed group named "blake"
//   collapseByLabel('blake')                 // collapse them again
//   setLevel(2)                              // bulk expand to depth 2
//   "document.getElementById('fit').click()" // press a toolbar button

import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";

const [url, ...rest] = process.argv.slice(2);
if (!url || rest.length % 2 !== 0) {
  console.error("usage: node drive.mjs <url> <out.png> <js> [<out.png> <js> ...]");
  process.exit(2);
}
const steps = [];
for (let i = 0; i < rest.length; i += 2) steps.push({ png: rest[i], js: rest[i + 1] });

const PORT = 9300 + Math.floor((Date.now() % 1000) / 10);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Helper functions injected into the page so steps can call them by name.
const HELPERS = `
window.selectCircuit = (i) => { const s=document.getElementById('circuit-select'); s.value=String(i); s.dispatchEvent(new Event('change')); };
window.expandByLabel = (lab) => cy.nodes().filter(n=>n.data('isGroup')&&n.data('label')===lab&&ec.isExpandable(n)).forEach(n=>n.emit('dbltap'));
window.collapseByLabel = (lab) => cy.nodes().filter(n=>n.data('isGroup')&&n.data('label')===lab&&ec.isCollapsible(n)).forEach(n=>n.emit('dbltap'));
window.groupSummary = () => cy.nodes().filter(n=>n.data('isGroup')).map(n=>n.data('label')+(ec.isExpandable(n)?'(+)':'(-)')).join(', ');
`;

const chrome = spawn(
  "google-chrome-stable",
  [
    "--headless=new", "--disable-gpu", "--no-sandbox", "--hide-scrollbars",
    `--remote-debugging-port=${PORT}`, "--window-size=1500,1000",
    `--user-data-dir=/tmp/cdp-${PORT}`, "about:blank",
  ],
  { stdio: "ignore" },
);

let nextId = 1;
const pending = new Map();
function send(ws, method, params = {}) {
  const id = nextId++;
  ws.send(JSON.stringify({ id, method, params }));
  return new Promise((res, rej) => pending.set(id, { res, rej }));
}
const evalJs = (ws, expression) =>
  // returnByValue:false — we only need side effects; serializing cy collections
  // (or functions) triggers "object reference chain too long".
  send(ws, "Runtime.evaluate", { expression, awaitPromise: true, returnByValue: false });

try {
  // Wait for the DevTools endpoint, then find the page target.
  let target;
  for (let i = 0; i < 50; i++) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json();
      target = list.find((t) => t.type === "page" && t.webSocketDebuggerUrl);
      if (target) break;
    } catch {}
    await sleep(100);
  }
  if (!target) throw new Error("no devtools page target");

  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) {
      const { res, rej } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? rej(new Error(JSON.stringify(msg.error))) : res(msg.result);
    }
  };

  await send(ws, "Page.enable");
  await send(ws, "Runtime.enable");
  await send(ws, "Page.navigate", { url });
  await sleep(2500); // load + cytoscape build + initial layout
  await evalJs(ws, HELPERS);

  for (const { png, js } of steps) {
    if (js && js.trim()) {
      const r = await evalJs(ws, js);
      if (r.exceptionDetails) console.error(`step js error: ${JSON.stringify(r.exceptionDetails)}`);
      await sleep(1200); // let layout/fit settle
    }
    const shot = await send(ws, "Page.captureScreenshot", { format: "png" });
    writeFileSync(png, Buffer.from(shot.data, "base64"));
    console.error(`wrote ${png}`);
  }
  ws.close();
} catch (e) {
  console.error("drive error:", e.message);
  process.exitCode = 1;
} finally {
  chrome.kill("SIGKILL");
}
