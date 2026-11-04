const host = document.getElementById("molkky-input");
const svg = host.querySelector("svg");
const listeners = new AbortController();
const options = { signal: listeners.signal };
let owner = null;
let frame;

function sample(event) {
    const rect = svg.getBoundingClientRect();
    const style = getComputedStyle(svg);
    const left = parseFloat(style.borderLeftWidth) || 0;
    const right = parseFloat(style.borderRightWidth) || 0;
    const top = parseFloat(style.borderTopWidth) || 0;
    const bottom = parseFloat(style.borderBottomWidth) || 0;
    const box = svg.viewBox.baseVal;
    return {
        pointer_id: event.pointerId, button: event.button,
        cursor: [event.clientX, event.clientY],
        rect: [rect.left + left, rect.top + top, rect.width - left - right, rect.height - top - bottom],
        bounds: [box.x, -box.y - box.height, box.width, box.height],
        time: performance.now(),
    };
}

function clearCapture() {
    const id = owner;
    owner = null;
    if (id !== null && svg.hasPointerCapture(id)) svg.releasePointerCapture(id);
}

function release(event) {
    if (event.pointerId !== owner || event.button !== 0) return;
    dioxus.send({ kind: "release", ...sample(event) });
    clearCapture();
}

svg.addEventListener("pointerdown", event => {
    if (event.pointerType !== "mouse" || event.button !== 0 || owner !== null || host.dataset.phase !== "ready") return;
    event.preventDefault();
    owner = event.pointerId;
    // ブラウザーがpointerを失う前に同期的にcaptureする。
    svg.setPointerCapture(owner);
    dioxus.send({ kind: "press", ...sample(event) });
}, options);

svg.addEventListener("pointermove", event => {
    if (event.pointerId !== owner) return;
    if (!(event.buttons & 1)) {
        if (event.button === 0) release(event);
        else {
            dioxus.send({ kind: "cancel", pointer_id: owner });
            clearCapture();
        }
        return;
    }
    dioxus.send({ kind: "move", ...sample(event) });
}, options);
svg.addEventListener("pointerup", release, options);

function cancel(event) {
    if (event.pointerId === owner) {
        dioxus.send({ kind: "cancel", pointer_id: owner });
        clearCapture();
    }
}
svg.addEventListener("pointercancel", cancel, options);
svg.addEventListener("lostpointercapture", cancel, options);
function blur() {
    clearCapture();
    dioxus.send({ kind: "blur" });
}
window.addEventListener("blur", blur, options);
document.addEventListener("visibilitychange", () => { if (document.hidden) blur(); }, options);

function tick(time) {
    if (host.dataset.phase === "charging" || host.dataset.phase === "simulating") {
        dioxus.send({ kind: "frame", time });
    }
    frame = requestAnimationFrame(tick);
}
frame = requestAnimationFrame(tick);
await dioxus.recv();
listeners.abort();
cancelAnimationFrame(frame);
clearCapture();
