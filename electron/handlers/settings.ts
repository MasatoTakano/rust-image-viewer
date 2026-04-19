import * as fs from "fs";
import * as path from "path";
import { app } from "electron";

function getSettingsPath(): string {
  const configDir = app.getPath("userData");
  fs.mkdirSync(configDir, { recursive: true });
  return path.join(configDir, "settings.json");
}

function createDefaults(): any {
  return {
    key_bindings: {
      next_page: ["ArrowLeft"],
      prev_page: ["ArrowRight"],
      toggle_fullscreen: ["Enter"],
      toggle_spread: ["Space"],
      go_first: ["Home"],
      go_last: ["End"],
      open_settings: ["Escape"],
    },
    window_size: { width: 1280, height: 900 },
    background_color: "#000000",
    preload_range: 10,
    key_throttle_ms: 40,
    wheel_throttle_ms: 150,
    display_mode: "single",
    resize_filter: "catmull_rom",
  };
}

export function loadSettings(): any {
  const settingsPath = getSettingsPath();
  if (!fs.existsSync(settingsPath)) {
    return createDefaults();
  }
  try {
    const content = fs.readFileSync(settingsPath, "utf-8");
    return { ...createDefaults(), ...JSON.parse(content) };
  } catch {
    return createDefaults();
  }
}

export function saveSettings(settings: any): void {
  const settingsPath = getSettingsPath();
  fs.writeFileSync(settingsPath, JSON.stringify(settings, null, 2), "utf-8");
}
