import { app, BrowserWindow, ipcMain, dialog } from "electron";
import * as path from "path";
import { loadSource } from "./handlers/fileLoader";
import { getImage } from "./handlers/imageProvider";
import { loadSettings, saveSettings } from "./handlers/settings";

let mainWindow: BrowserWindow | null = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 900,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: false,
      nodeIntegration: false,
      sandbox: false,
    },
    title: "Rust Image Viewer",
    show: false,
    backgroundColor: "#000000",
  });

  if (process.env.ELECTRON_DEV) {
    mainWindow.loadURL("http://localhost:5173");
  } else {
    mainWindow.loadFile(path.join(__dirname, "..", "dist", "index.html"));
  }

  mainWindow.once("ready-to-show", () => {
    mainWindow?.show();
  });

  mainWindow.on("closed", () => {
    mainWindow = null;
  });
}

function registerIpcHandlers() {
  ipcMain.handle("load-source", async (_event, p: string) => {
    return loadSource(p);
  });

  ipcMain.handle(
    "get-image",
    async (_event, entry: any, maxWidth: number, maxHeight: number, filterType?: string) => {
      try {
        const result = await getImage(entry, maxWidth, maxHeight, filterType);
        return result;
      } catch (error) {
        console.error("get-image error:", error);
        throw error;
      }
    },
  );

  ipcMain.handle("load-settings", async () => {
    return loadSettings();
  });

  ipcMain.handle("save-settings", async (_event, settings: any) => {
    saveSettings(settings);
  });

  ipcMain.handle(
    "open-dialog",
    async (_event, options: { directory?: boolean; multiple?: boolean; title?: string; filters?: any[] }) => {
      const result = await dialog.showOpenDialog(mainWindow!, {
        title: options.title,
        properties: [
          options.directory ? "openDirectory" : "openFile",
          ...(options.multiple ? ["multiSelections" as const] : []),
        ],
        filters: options.filters,
      });
      if (result.canceled || result.filePaths.length === 0) return null;
      return result.filePaths[0];
    },
  );

  ipcMain.handle("set-fullscreen", async (_event, fullscreen: boolean) => {
    mainWindow?.setFullScreen(fullscreen);
  });

  ipcMain.handle("is-fullscreen", async () => {
    return mainWindow?.isFullScreen() ?? false;
  });

  ipcMain.handle("set-window-size", async (_event, width: number, height: number) => {
    mainWindow?.setContentSize(width, height);
  });
}

app.whenReady().then(() => {
  registerIpcHandlers();
  createWindow();

  const cliArgs = process.argv.slice(1).filter((a) => !a.startsWith("--") && a !== ".");
  if (cliArgs.length > 0) {
    const filePath = cliArgs[0];
    setTimeout(() => {
      mainWindow?.webContents.send("cli-file-open", filePath);
    }, 1000);
  }

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
