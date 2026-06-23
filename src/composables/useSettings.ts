import { ref, readonly } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types";

const settings = ref<AppSettings>(createDefaults());

function createDefaults(): AppSettings {
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
    window_state: { x: null, y: null, width: 1280, height: 900 },
    background_color: "#000000",
    preload_range: 10,
    key_throttle_ms: 80,
    wheel_throttle_ms: 150,
    display_mode: "single",
  };
}

export function useSettings() {
  async function load(): Promise<AppSettings> {
    try {
      const s = await invoke<AppSettings>("load_settings");
      settings.value = s;
      return s;
    } catch {
      const defaults = createDefaults();
      settings.value = defaults;
      return defaults;
    }
  }

  async function save(s: AppSettings) {
    await invoke("save_settings", { settings: s });
    settings.value = s;
  }

  return {
    settings: readonly(settings),
    load,
    save,
  };
}
