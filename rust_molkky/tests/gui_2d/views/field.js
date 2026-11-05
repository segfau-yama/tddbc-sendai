const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { test } = require("node:test");
const { runInNewContext } = require("node:vm");
const script = readFileSync(join(__dirname, "../../../src/gui_2d/views/field.js"), "utf8");

for (const failed of [false, true]) {
    test(failed ? "接続失敗でもフレームとcaptureを解除し入力を取り消す" : "破棄時にフレームとcaptureを解除し入力を取り消す", async () => {
        const input = new EventTarget(), document = new EventTarget(), window = new EventTarget();
        const svg = { querySelector: () => input };
        const host = { dataset: { phase: "ready" }, querySelector: () => svg };
        document.getElementById = () => host;
        let capture = null;
        input.setPointerCapture = id => { capture = id; };
        input.hasPointerCapture = id => capture === id;
        input.releasePointerCapture = () => { capture = null; };
        const frames = new Set(), sent = [], canceled = [];
        input.addEventListener("pointercancel", event => canceled.push(event.pointerId));
        class PointerEvent extends Event {
            constructor(type, options) { super(type, options); this.pointerId = options.pointerId; }
        }
        let stop;
        const receive = new Promise((resolve, reject) => { stop = failed ? () => reject(new Error("connection closed")) : resolve; });
        const running = runInNewContext(`(async () => { ${script} })()`, {
            document, window, AbortController, PointerEvent,
            requestAnimationFrame: callback => { frames.add(callback); return callback; },
            cancelAnimationFrame: callback => frames.delete(callback),
            dioxus: { send: value => sent.push(value), recv: () => receive },
        });
        function press() {
            input.dispatchEvent(Object.assign(new Event("pointerdown", { cancelable: true }), {
                pointerType: "mouse", pointerId: 1, button: 0,
            }));
        }
        press();
        assert.equal(capture, 1);
        assert.equal(frames.size, 1);
        assert.equal(sent.length, 0); // マウス入力はDioxus側が処理し、JSから転送しない。
        const frame = [...frames][0];
        frames.delete(frame);
        frame(123);
        assert.equal(sent[0], 123);
        stop();
        if (failed) await assert.rejects(running, /connection closed/);
        else await running;
        assert.equal(capture, null);
        assert.equal(frames.size, 0);
        assert.deepEqual(canceled, [1]);
        press();
        window.dispatchEvent(new Event("blur"));
        assert.equal(capture, null);
        assert.equal(sent.length, 1);
    });
}
