const endpoint = await dioxus.recv();
const surface = document.getElementById("remote-native-surface");
if (!surface) return;
const videoRect = () => {
    const bounds = surface.getBoundingClientRect();
    if (surface.dataset.original === "true") return bounds;
    const scale = Math.min(bounds.width / Number(surface.dataset.width), bounds.height / Number(surface.dataset.height));
    const width = Number(surface.dataset.width) * scale, height = Number(surface.dataset.height) * scale;
    return { left: bounds.left + (bounds.width-width)/2, top: bounds.top + (bounds.height-height)/2, width, height };
};
const events = new AbortController();
const options = { signal: events.signal, passive: false };
const modifiers = e => ({ alt: !!e.altKey, ctrl: !!e.ctrlKey, shift: !!e.shiftKey, meta: !!e.metaKey });
let buttons = 0, position = { x: 0, y: 0 }, stopped = false;
const keys = new Map();
const mouse = (mask, e, point = position) => dioxus.send({ kind: "mouse", mask, ...point, ...modifiers(e) });
const key = (e, down) => dioxus.send({ kind: "key", key: e.key, down, ...modifiers(e) });
const releaseButtons = () => {
    for (const button of [1, 2, 4, 8, 16]) if (buttons & button) mouse(button << 3 | 2, {});
    buttons = 0;
};
const release = () => {
    releaseButtons();
    for (const e of keys.values()) key({ key: e.key }, false);
    keys.clear();
};
surface.addEventListener("pointerdown", e => {
    if (!["mouse", "pen", "touch"].includes(e.pointerType)) return;
    const bounds = videoRect();
    if (e.clientX < bounds.left || e.clientX > bounds.left + bounds.width || e.clientY < bounds.top || e.clientY > bounds.top + bounds.height) return;
    e.preventDefault();
    surface.focus({ preventScroll: true });
    surface.setPointerCapture(e.pointerId);
    const rect = videoRect();
    position = { x: (e.clientX - rect.left) / rect.width, y: (e.clientY - rect.top) / rect.height };
    const button = [1, 4, 2, 8, 16][e.button] || 1;
    buttons |= button;
    mouse(button << 3 | 1, e);
}, options);
surface.addEventListener("pointermove", e => {
    const rect = videoRect();
    if (!buttons && (e.clientX < rect.left || e.clientX > rect.left + rect.width || e.clientY < rect.top || e.clientY > rect.top + rect.height)) return;
    position = { x: (e.clientX - rect.left) / rect.width, y: (e.clientY - rect.top) / rect.height };
    mouse(buttons << 3, e);
}, options);
surface.addEventListener("pointerup", e => {
    const rect = videoRect();
    position = { x: (e.clientX - rect.left) / rect.width, y: (e.clientY - rect.top) / rect.height };
    const button = [1, 4, 2, 8, 16][e.button] || 1;
    buttons &= ~button;
    mouse(button << 3 | 2, e);
}, options);
surface.addEventListener("pointercancel", releaseButtons, options);
surface.addEventListener("lostpointercapture", releaseButtons, options);
surface.addEventListener("blur", release, options);
window.addEventListener("blur", release, options);
surface.addEventListener("contextmenu", e => e.preventDefault(), options);
surface.addEventListener("wheel", e => {
    e.preventDefault();
    const unit = e.deltaMode === 0 ? 40 : 1;
    mouse(3, e, { x: -Math.round(e.deltaX / unit), y: -Math.round(e.deltaY / unit) });
}, options);
surface.addEventListener("keydown", e => {
    e.preventDefault();
    keys.set(e.code, { key: e.key });
    key(e, true);
}, options);
surface.addEventListener("keyup", e => {
    e.preventDefault();
    key({ key: keys.get(e.code)?.key || e.key, altKey: e.altKey, ctrlKey: e.ctrlKey, shiftKey: e.shiftKey, metaKey: e.metaKey }, false);
    keys.delete(e.code);
}, options);

const overlay = document.createElement("img");
Object.assign(overlay.style, {position:"fixed",pointerEvents:"none",zIndex:"10",display:"none"});
surface.after(overlay);
let lastError = 0;
const report = error => {
    if (performance.now() - lastError > 10000 || lastError === 0) {
        console.error("Native remote surface:", error);
        dioxus.send({ kind: "fault", message: String(error) });
        lastError = performance.now();
    }
};
let geometryFrame = 0, lastGeometry = "";
const geometry = () => {
    if (geometryFrame || stopped) return;
    geometryFrame = requestAnimationFrame(() => {
        geometryFrame = 0;
        const rect = videoRect();
        let left = 0, top = 0, right = innerWidth, bottom = innerHeight;
        for (let parent = surface.parentElement; parent; parent = parent.parentElement) {
            const css = getComputedStyle(parent), box = parent.getBoundingClientRect();
            if (/(auto|scroll|hidden|clip)/.test(css.overflowX)) { left = Math.max(left,box.left); right = Math.min(right,box.right); }
            if (/(auto|scroll|hidden|clip)/.test(css.overflowY)) { top = Math.max(top,box.top); bottom = Math.min(bottom,box.bottom); }
        }
        const viewport = { rect:[rect.left,rect.top,rect.width,rect.height],clip:[left,top,Math.max(0,right-left),Math.max(0,bottom-top)],window:[innerWidth,innerHeight] };
        const serialized = JSON.stringify(viewport);
        if (serialized !== lastGeometry) { lastGeometry = serialized; dioxus.send({viewport}); }
    });
};
const resize = new ResizeObserver(geometry);
resize.observe(surface);resize.observe(document.documentElement);
window.addEventListener("resize",geometry,options);
window.addEventListener("scroll",geometry,{signal:events.signal,capture:true});
geometry();
let cursorKey = "";
const cursor = async () => {
    if (stopped) return;
    try {
        const response = await fetch(`${endpoint}/cursor`, { signal: events.signal });
        if (response.status === 200) {
            const state = await response.json(), shape = state.shape;
            const rect = videoRect(), scale = state.zoom ? rect.width / Number(surface.dataset.width) : 1;
            const visible = state.show && !state.embedded && shape && state.position;
            overlay.style.display = "none";
            if (visible) {
                const x = (state.position[0] - state.origin[0]) / Number(surface.dataset.width), y = (state.position[1] - state.origin[1]) / Number(surface.dataset.height);
                if (x >= 0 && x < 1 && y >= 0 && y < 1) {
                    overlay.src = shape.uri;
                    Object.assign(overlay.style, { display: "block", left: `${rect.left + x * rect.width - shape.hotx * scale}px`, top: `${rect.top + y * rect.height - shape.hoty * scale}px`, width: `${shape.width * scale}px`, height: `${shape.height * scale}px` });
                }
            }
            if (state.blocked) { surface.style.cursor = "not-allowed"; cursorKey = ""; }
            else if (state.embedded || overlay.style.display !== "none") { surface.style.cursor = "none"; cursorKey = ""; }
            else if (shape) {
                const cacheKey = `${shape.id}:${scale}`;
                if (cacheKey !== cursorKey) {
                    const width = Math.max(1,Math.min(128,Math.round(shape.width*scale))), height = Math.max(1,Math.min(128,Math.round(shape.height*scale)));
                    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"><image href="${shape.uri}" width="${width}" height="${height}"/></svg>`;
                    surface.style.cursor = `url("data:image/svg+xml,${encodeURIComponent(svg)}") ${Math.min(width-1,Math.round(shape.hotx*width/shape.width))} ${Math.min(height-1,Math.round(shape.hoty*height/shape.height))}, default`;
                    cursorKey = cacheKey;
                }
            } else { surface.style.cursor = "default"; cursorKey = ""; }
        }
    } catch (error) { if (!stopped) report(error); }
    if (!stopped) setTimeout(cursor, 50);
};
cursor();
await new Promise(resolve => {
    const observer = new MutationObserver(() => {
        if (surface.isConnected) { geometry(); return; }
        release(); stopped = true; events.abort(); resize.disconnect(); cancelAnimationFrame(geometryFrame); overlay.remove(); observer.disconnect(); resolve();
    });
    observer.observe(document.body, {childList:true,subtree:true,attributes:true,attributeFilter:["data-width","data-height","data-original"]});
});
