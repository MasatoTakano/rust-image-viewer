const { ipcRenderer, webUtils } = require("electron");

// @ts-check
/** @type {any} */
const api = {
  loadSource: (/** @type {string} */ p) => ipcRenderer.invoke("load-source", p),
  getImage: (/** @type {any} */ entry, /** @type {number} */ maxWidth, /** @type {number} */ maxHeight, /** @type {string|undefined} */ filterType) =>
    ipcRenderer.invoke("get-image", entry, maxWidth, maxHeight, filterType),
  loadSettings: () => ipcRenderer.invoke("load-settings"),
  saveSettings: (/** @type {any} */ settings) => ipcRenderer.invoke("save-settings", settings),
  openDialog: (/** @type {any} */ options) => ipcRenderer.invoke("open-dialog", options),
  setFullscreen: (/** @type {boolean} */ fullscreen) => ipcRenderer.invoke("set-fullscreen", fullscreen),
  isFullscreen: () => ipcRenderer.invoke("is-fullscreen"),
  setWindowSize: (/** @type {number} */ width, /** @type {number} */ height) =>
    ipcRenderer.invoke("set-window-size", width, height),
  onCliFileOpen: (/** @type {(p: string) => void} */ callback) => {
    const handler = (/** @type {any} */ _event, /** @type {string} */ p) => callback(p);
    ipcRenderer.on("cli-file-open", handler);
    return () => ipcRenderer.removeListener("cli-file-open", handler);
  },
  onFileDropped: null,
};

let _fileDropCallback = null;
api.onFileDropped = (/** @type {((p: string) => void) | null} */ cb) => {
  _fileDropCallback = cb;
};

let _dragStateCallback = null;
api.onDragStateChanged = null;
api.onDragStateChanged = (/** @type {((over: boolean) => void) | null} */ cb) => {
  _dragStateCallback = cb;
};

window.electronAPI = api;

document.addEventListener("dragover", (/** @type {DragEvent} */ e) => {
  e.preventDefault();
});

document.addEventListener("drop", async (/** @type {DragEvent} */ e) => {
  e.preventDefault();
  e.stopPropagation();
  if (_dragStateCallback) _dragStateCallback(false);
  const dt = e.dataTransfer;
  if (!dt || !dt.files || dt.files.length === 0) return;
  const file = dt.files[0];
  try {
    const filePath = webUtils.getPathForFile(file);
    if (_fileDropCallback) {
      _fileDropCallback(filePath);
    }
  } catch (err) {
    console.error("[preload] drop handling failed:", err);
  }
});

document.addEventListener("dragenter", (/** @type {DragEvent} */ e) => {
  if (_dragStateCallback) _dragStateCallback(true);
});

document.addEventListener("dragleave", (/** @type {DragEvent} */ e) => {
  if (_dragStateCallback) _dragStateCallback(false);
});
