import { onMounted, onUnmounted } from "vue";
import type { KeyBindings } from "../types";
import { usePageController } from "./usePageController";
import { useImageStore } from "./useImageStore";
import { useSettings } from "./useSettings";
import { useFullscreen } from "./useFullscreen";
import { buildKeyCombo } from "../utils/keyCombo";

export function useKeyBindings() {
  const { nextPage, prevPage, goFirst, goLast, toggleDisplayMode, currentIndex } = usePageController();
  const { refreshInBackground } = useImageStore();
  const { settings } = useSettings();
  const { setFullscreen } = useFullscreen();

  let keyBindings: KeyBindings | null = null;
  let onToggleSettings: (() => void) | null = null;
  let isSettingsOpen: (() => boolean) | null = null;
  let lastActionTime = 0;

  function setKeyBindings(bindings: KeyBindings) {
    keyBindings = bindings;
  }

  function setOnToggleSettings(cb: () => void) {
    onToggleSettings = cb;
  }

  function setSettingsOpenChecker(checker: () => boolean) {
    isSettingsOpen = checker;
  }

  function throttleAction(fn: () => void) {
    const now = Date.now();
    if (now - lastActionTime < settings.value.key_throttle_ms) return;
    lastActionTime = now;
    fn();
  }

  function matchesBinding(combo: string, bindings: readonly string[]): boolean {
    return bindings.some((b) => combo === b || combo.endsWith("+" + b));
  }

  async function toggleFullscreen() {
    try {
      const is = await window.electronAPI.isFullscreen();
      await window.electronAPI.setFullscreen(!is);
      setFullscreen(!is);
      refreshInBackground(currentIndex.value);
    } catch (error) {
      console.error("フルスクリーン切替エラー:", error);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!keyBindings) return;

    if (isSettingsOpen?.()) return;

    const combo = buildKeyCombo(e);

    if (matchesBinding(combo, keyBindings.next_page)) {
      e.preventDefault();
      throttleAction(nextPage);
    } else if (matchesBinding(combo, keyBindings.prev_page)) {
      e.preventDefault();
      throttleAction(prevPage);
    } else if (matchesBinding(combo, keyBindings.toggle_fullscreen)) {
      e.preventDefault();
      toggleFullscreen();
    } else if (matchesBinding(combo, keyBindings.toggle_spread)) {
      e.preventDefault();
      toggleDisplayMode();
    } else if (matchesBinding(combo, keyBindings.go_first)) {
      e.preventDefault();
      goFirst();
    } else if (matchesBinding(combo, keyBindings.go_last)) {
      e.preventDefault();
      goLast();
    } else if (matchesBinding(combo, keyBindings.open_settings)) {
      e.preventDefault();
      onToggleSettings?.();
    }
  }

  onMounted(() => document.addEventListener("keydown", handleKeyDown));
  onUnmounted(() => document.removeEventListener("keydown", handleKeyDown));

  return { setKeyBindings, setOnToggleSettings, setSettingsOpenChecker };
}
