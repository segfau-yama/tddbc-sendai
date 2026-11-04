const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { test } = require("node:test");
const { runInNewContext } = require("node:vm");

const script = readFileSync(join(__dirname, "../../../src/gui_2d/views/app.js"), "utf8");

for (const failed of [false, true]) {
    test(failed ? "接続失敗でもイベントとフレームとcaptureを解除する" : "破棄時にイベントとフレームとcaptureを解除する", async () => {
        const svg = new EventTarget();
        const document = new EventTarget();
        const window = new EventTarget();
        const host = { dataset: { phase: "ready" }, querySelector: () => svg };
        document.getElementById = () => host;
        svg.viewBox = { baseVal: { x: -0.3, y: -1.35, width: 0.6, height: 1.5 } };
        svg.getBoundingClientRect = () => ({ left: 0, top: 0, width: 600, height: 1500 });
        let capture = null;
        svg.setPointerCapture = id => { capture = id; };
        svg.hasPointerCapture = id => capture === id;
        svg.releasePointerCapture = () => { capture = null; };
        const frames = new Set();
        const sent = [];
        let stop;
        const receive = new Promise((resolve, reject) => { stop = failed ? () => reject(new Error("connection closed")) : resolve; });
        const running = runInNewContext(`(async () => { ${script} })()`, {
            document, window, AbortController,
            getComputedStyle: () => ({}),
            performance: { now: () => 100 },
            requestAnimationFrame: callback => { frames.add(callback); return callback; },
            cancelAnimationFrame: callback => frames.delete(callback),
            dioxus: { send: value => sent.push(value), recv: () => receive },
        });
        function press() {
            svg.dispatchEvent(Object.assign(new Event("pointerdown", { cancelable: true }), {
                pointerType: "mouse", pointerId: 1, button: 0, clientX: 300, clientY: 350,
            }));
        }
        press();
        assert.equal(capture, 1);
        assert.equal(frames.size, 1);
        assert.equal(sent.length, 1);
        stop();
        if (failed) await assert.rejects(running, /connection closed/);
        else await running;
        assert.equal(capture, null);
        assert.equal(frames.size, 0);
        press();
        window.dispatchEvent(new Event("blur"));
        assert.equal(sent.length, 1);
        assert.equal(capture, null);
    });
}
