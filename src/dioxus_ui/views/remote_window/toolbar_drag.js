const handle = document.getElementById("remote-toolbar-drag");
const toolbar = document.getElementById("remote-toolbar");
const area = document.getElementById("remote-toolbar-area");
if (!handle || !toolbar || !area) return;
const events = new AbortController();
let drag;
const send = (e, finished) => {
    const parent = area.getBoundingClientRect();
    dioxus.send({ cursor: [e.clientX - parent.left, e.clientY - parent.top], parent: [parent.width, parent.height], size: drag.size, grab: drag.grab, finished });
};
handle.addEventListener("pointerdown", e => {
    if (e.button !== 0) return;
    e.preventDefault();
    handle.setPointerCapture(e.pointerId);
    const rect = toolbar.getBoundingClientRect();
    drag = { size: [rect.width, rect.height], grab: rect.width >= rect.height ? e.clientX - rect.left : e.clientY - rect.top, x: e.clientX, y: e.clientY };
    send(e, false);
}, { signal: events.signal });
handle.addEventListener("pointermove", e => {
    if (!drag) return;
    drag.x = e.clientX; drag.y = e.clientY;
    send(e, false);
}, { signal: events.signal });
const finish = () => {
    if (!drag) return;
    send({ clientX: drag.x, clientY: drag.y }, true);
    drag = null;
};
handle.addEventListener("pointerup", finish, { signal: events.signal });
handle.addEventListener("lostpointercapture", finish, { signal: events.signal });
handle.addEventListener("pointercancel", finish, { signal: events.signal });
window.addEventListener("blur", finish, { signal: events.signal });
await new Promise(resolve => {
    const observer = new MutationObserver(() => {
        if (handle.isConnected) return;
        finish(); events.abort(); observer.disconnect(); resolve();
    });
    observer.observe(area, { subtree: true, childList: true });
});
