const endpoint = await dioxus.recv();
const canvas = document.getElementById("remote-desktop");
if (!canvas) return;
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
canvas.addEventListener("pointerdown", e => {
    if (!["mouse", "pen", "touch"].includes(e.pointerType)) return;
    e.preventDefault();
    canvas.focus({ preventScroll: true });
    canvas.setPointerCapture(e.pointerId);
    const rect = canvas.getBoundingClientRect();
    position = { x: (e.clientX - rect.left) / rect.width, y: (e.clientY - rect.top) / rect.height };
    const button = [1, 4, 2, 8, 16][e.button] || 1;
    buttons |= button;
    mouse(button << 3 | 1, e);
}, options);
canvas.addEventListener("pointermove", e => {
    const rect = canvas.getBoundingClientRect();
    position = { x: (e.clientX - rect.left) / rect.width, y: (e.clientY - rect.top) / rect.height };
    mouse(buttons << 3, e);
}, options);
canvas.addEventListener("pointerup", e => {
    const rect = canvas.getBoundingClientRect();
    position = { x: (e.clientX - rect.left) / rect.width, y: (e.clientY - rect.top) / rect.height };
    const button = [1, 4, 2, 8, 16][e.button] || 1;
    buttons &= ~button;
    mouse(button << 3 | 2, e);
}, options);
canvas.addEventListener("pointercancel", releaseButtons, options);
canvas.addEventListener("lostpointercapture", releaseButtons, options);
canvas.addEventListener("blur", release, options);
window.addEventListener("blur", release, options);
canvas.addEventListener("contextmenu", e => e.preventDefault(), options);
canvas.addEventListener("wheel", e => {
    e.preventDefault();
    const unit = e.deltaMode === 0 ? 40 : 1;
    mouse(3, e, { x: -Math.round(e.deltaX / unit), y: -Math.round(e.deltaY / unit) });
}, options);
canvas.addEventListener("keydown", e => {
    e.preventDefault();
    keys.set(e.code, { key: e.key });
    key(e, true);
}, options);
canvas.addEventListener("keyup", e => {
    e.preventDefault();
    key({ key: keys.get(e.code)?.key || e.key, altKey: e.altKey, ctrlKey: e.ctrlKey, shiftKey: e.shiftKey, metaKey: e.metaKey }, false);
    keys.delete(e.code);
}, options);

const overlay = document.createElement("img");
Object.assign(overlay.style, { position: "fixed", pointerEvents: "none", zIndex: "10", display: "none" });
canvas.after(overlay);
const gl = canvas.getContext("webgl", { alpha: false, antialias: false, depth: false, stencil: false });
let draw;
if (gl) {
    const shader = (type, source) => {
        const result = gl.createShader(type);
        gl.shaderSource(result, source);
        gl.compileShader(result);
        if (!gl.getShaderParameter(result, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(result));
        return result;
    };
    const program = gl.createProgram();
    gl.attachShader(program, shader(gl.VERTEX_SHADER, "attribute vec2 p; varying vec2 uv; void main(){uv=vec2((p.x+1.0)*0.5,(1.0-p.y)*0.5);gl_Position=vec4(p,0,1);}"));
    gl.attachShader(program, shader(gl.FRAGMENT_SHADER, "precision mediump float; varying vec2 uv; uniform sampler2D frame; uniform float rowScale; uniform bool bgra; void main(){vec4 c=texture2D(frame,vec2(uv.x*rowScale,uv.y));gl_FragColor=bgra?c.bgra:c;}"));
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program));
    gl.useProgram(program);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1, 1,-1, -1,1, 1,1]), gl.STATIC_DRAW);
    const p = gl.getAttribLocation(program, "p");
    gl.enableVertexAttribArray(p);
    gl.vertexAttribPointer(p, 2, gl.FLOAT, false, 0, 0);
    gl.bindTexture(gl.TEXTURE_2D, gl.createTexture());
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    const rowScale = gl.getUniformLocation(program, "rowScale"), bgra = gl.getUniformLocation(program, "bgra");
    const maxTexture = gl.getParameter(gl.MAX_TEXTURE_SIZE);
    let lastWidth = 0, lastHeight = 0, lastFormat;
    draw = (pixels, width, height, stride, format) => {
        if (width > maxTexture || height > maxTexture) throw new Error(`Remote frame ${width}x${height} exceeds WebGL texture size ${maxTexture}`);
        const bpp = format === "rgb" ? 3 : 4;
        let textureWidth = stride / bpp;
        if (!Number.isInteger(textureWidth) || textureWidth > maxTexture) {
            const packed = new Uint8Array(width * height * bpp);
            for (let y = 0; y < height; y++) packed.set(pixels.subarray(y * stride, y * stride + width * bpp), y * width * bpp);
            pixels = packed;
            textureWidth = width;
        }
        if (canvas.width !== width) canvas.width = width;
        if (canvas.height !== height) canvas.height = height;
        gl.viewport(0, 0, width, height);
        const pixelFormat = bpp === 3 ? gl.RGB : gl.RGBA;
        if (textureWidth !== lastWidth || height !== lastHeight || pixelFormat !== lastFormat) {
            gl.texImage2D(gl.TEXTURE_2D, 0, pixelFormat, textureWidth, height, 0, pixelFormat, gl.UNSIGNED_BYTE, pixels);
            lastWidth = textureWidth; lastHeight = height; lastFormat = pixelFormat;
        } else gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, textureWidth, height, pixelFormat, gl.UNSIGNED_BYTE, pixels);
        gl.uniform1f(rowScale, width / textureWidth);
        gl.uniform1i(bgra, format === "bgra");
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    };
} else {
    const ctx = canvas.getContext("2d", { alpha: false });
    draw = (pixels, width, height, stride, format) => {
        const rgba = new Uint8ClampedArray(width * height * 4), bpp = format === "rgb" ? 3 : 4;
        for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
            const src = y * stride + x * bpp, dst = (y * width + x) * 4;
            rgba[dst] = pixels[src + (format === "bgra" ? 2 : 0)];
            rgba[dst + 1] = pixels[src + 1]; rgba[dst + 2] = pixels[src + (format === "bgra" ? 0 : 2)]; rgba[dst + 3] = 255;
        }
        if (canvas.width !== width) canvas.width = width;
        if (canvas.height !== height) canvas.height = height;
        ctx.putImageData(new ImageData(rgba, width, height), 0, 0);
    };
}
let lastError = 0;
const report = error => {
    if (performance.now() - lastError > 10000 || lastError === 0) {
        console.error("Remote canvas:", error);
        dioxus.send({ kind: "fault", message: String(error) });
        lastError = performance.now();
    }
};
const render = async () => {
    if (stopped) return;
    try {
        const response = await fetch(`${endpoint}/frame`, { signal: events.signal });
        if (response.status === 200) {
            const value = name => Number(response.headers.get(`x-frame-${name}`));
            draw(new Uint8Array(await response.arrayBuffer()), value("width"), value("height"), value("stride"), response.headers.get("x-frame-format"));
        } else if (response.status !== 204) throw new Error(`Frame response: ${response.status}`);
    } catch (error) { if (!stopped) report(error); }
    if (!stopped) requestAnimationFrame(render);
};
let cursorKey = "";
const cursor = async () => {
    if (stopped) return;
    try {
        const response = await fetch(`${endpoint}/cursor`, { signal: events.signal });
        if (response.status === 200) {
            const state = await response.json(), shape = state.shape;
            const rect = canvas.getBoundingClientRect(), scale = state.zoom ? rect.width / canvas.width : 1;
            const visible = state.show && !state.embedded && shape && state.position;
            overlay.style.display = "none";
            if (visible) {
                const x = (state.position[0] - state.origin[0]) / canvas.width, y = (state.position[1] - state.origin[1]) / canvas.height;
                if (x >= 0 && x < 1 && y >= 0 && y < 1) {
                    overlay.src = shape.uri;
                    Object.assign(overlay.style, { display: "block", left: `${rect.left + x * rect.width - shape.hotx * scale}px`, top: `${rect.top + y * rect.height - shape.hoty * scale}px`, width: `${shape.width * scale}px`, height: `${shape.height * scale}px` });
                }
            }
            if (state.blocked) { canvas.style.cursor = "not-allowed"; cursorKey = ""; }
            else if (state.embedded || overlay.style.display !== "none") { canvas.style.cursor = "none"; cursorKey = ""; }
            else if (shape) {
                const cacheKey = `${shape.id}:${scale}`;
                if (cacheKey !== cursorKey) {
                    const image = new Image(); image.src = shape.uri; await image.decode();
                    const scaled = document.createElement("canvas");
                    scaled.width = Math.max(1, Math.min(128, Math.round(shape.width * scale)));
                    scaled.height = Math.max(1, Math.min(128, Math.round(shape.height * scale)));
                    scaled.getContext("2d").drawImage(image, 0, 0, scaled.width, scaled.height);
                    canvas.style.cursor = `url("${scaled.toDataURL()}") ${Math.min(scaled.width - 1, Math.round(shape.hotx * scaled.width / shape.width))} ${Math.min(scaled.height - 1, Math.round(shape.hoty * scaled.height / shape.height))}, default`;
                    cursorKey = cacheKey;
                }
            } else { canvas.style.cursor = "default"; cursorKey = ""; }
        }
    } catch (error) { if (!stopped) report(error); }
    if (!stopped) setTimeout(cursor, 50);
};
render();
cursor();
await new Promise(resolve => {
    const observer = new MutationObserver(() => {
        if (canvas.isConnected) return;
        release(); stopped = true; events.abort(); overlay.remove(); observer.disconnect(); resolve();
    });
    observer.observe(document.body, { childList: true, subtree: true });
});
