const host = document.getElementById("molkky-input");
const svg = host.querySelector("svg");
const input = svg.querySelector("[data-stick-input]");
const listeners = new AbortController();
const options = { signal: listeners.signal };
let owner = null;
let frame;

// 共通Dioxus APIにない同期captureだけを補う。入力処理はRustのポインターイベント。
input.addEventListener("pointerdown", event => {
    if (event.pointerType !== "mouse" || event.button !== 0 || owner !== null || host.dataset.phase !== "ready") return;
    event.preventDefault();
    owner = event.pointerId;
    input.setPointerCapture(owner);
}, options);
input.addEventListener("pointerup", event => {
    if (event.pointerId === owner && event.button === 0) owner = null;
}, options);
input.addEventListener("lostpointercapture", event => {
    if (event.pointerId === owner) owner = null;
}, options);
function cancel() {
    const id = owner;
    owner = null;
    if (id === null) return;
    input.dispatchEvent(new PointerEvent("pointercancel", { pointerId: id, bubbles: true }));
    if (input.hasPointerCapture(id)) input.releasePointerCapture(id);
}
window.addEventListener("blur", cancel, options);
document.addEventListener("visibilitychange", () => { if (document.hidden) cancel(); }, options);
function tick(time) {
    dioxus.send(time);
    frame = requestAnimationFrame(tick);
}
frame = requestAnimationFrame(tick);
try { await dioxus.recv(); }
finally {
    listeners.abort();
    cancelAnimationFrame(frame);
    cancel();
}
