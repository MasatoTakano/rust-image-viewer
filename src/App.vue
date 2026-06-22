<template>
  <div id="app" :style="{ backgroundColor: settings.settings.value.background_color }">
    <DropZone
      v-if="!hasEntries"
      @path-selected="loadFromPath"
      @notify="showNotification"
    />
    <template v-else>
      <ImageViewer ref="viewerRef" />
      <PageSlider v-if="!isFullscreen" />
      <StatusBar v-if="!isFullscreen" />
    </template>
    <SettingsPanel
      :visible="settingsVisible"
      @close="settingsVisible = false"
      @saved="onSettingsSaved"
      @notify="showNotification"
    />
    <Notification :message="notification" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import DropZone from "./components/DropZone.vue";
import ImageViewer from "./components/ImageViewer.vue";
import StatusBar from "./components/StatusBar.vue";
import PageSlider from "./components/PageSlider.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import Notification from "./components/Notification.vue";
import type { ImageEntry, DisplayMode } from "./types";
import { useImageStore } from "./composables/useImageStore";
import { useSettings } from "./composables/useSettings";
import { useKeyBindings } from "./composables/useKeyBindings";
import { useFullscreen } from "./composables/useFullscreen";
import { usePageController } from "./composables/usePageController";
import { useDragDrop } from "./composables/useDragDrop";

const { entries, setEntries, preloadAround } = useImageStore();
const { setDisplayMode, displayMode, resetIndex } = usePageController();
const settings = useSettings();
const { setKeyBindings, setOnToggleSettings, setSettingsOpenChecker } = useKeyBindings();
const { isFullscreen, setFullscreen } = useFullscreen();

const viewerRef = ref<InstanceType<typeof ImageViewer> | null>(null);
const settingsVisible = ref(false);
const notification = ref("");
let notificationTimer: ReturnType<typeof setTimeout> | null = null;

const hasEntries = computed(() => entries.value.length > 0);

function showNotification(msg: string) {
  notification.value = msg;
  if (notificationTimer) clearTimeout(notificationTimer);
  notificationTimer = setTimeout(() => {
    notification.value = "";
  }, 2000);
}

function onLoaded(list: ImageEntry[]) {
  setEntries(list);
  resetIndex();
  preloadAround(0);
}

const { start: startDragDrop, loadFromPath } = useDragDrop(onLoaded, showNotification);

function onSettingsSaved() {
  setKeyBindings(settings.settings.value.key_bindings);
}

async function toggleSettings() {
  if (isFullscreen.value) {
    try { await getCurrentWindow().setFullscreen(false); } catch {}
    setFullscreen(false);
  }
  settingsVisible.value = !settingsVisible.value;
}

let resizeTimer: ReturnType<typeof setTimeout> | null = null;
let unlistenCli: (() => void) | null = null;
let unlistenDragDrop: (() => void) | null = null;

function onResize() {
  if (resizeTimer) clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    viewerRef.value?.refreshAfterResize();
  }, 50);
}

function onSaveState() {
  const currentSettings = settings.settings.value;
  const newSettings = {
    ...currentSettings,
    window_size: { width: window.innerWidth, height: window.innerHeight },
    display_mode: displayMode.value,
  };
  invoke("save_settings", { settings: newSettings }).catch(() => {});
}

onMounted(async () => {
  const s = await settings.load();
  setKeyBindings(s.key_bindings);

  if (s.display_mode === "spread" || s.display_mode === "single") {
    setDisplayMode(s.display_mode as DisplayMode);
  }

  try {
    const { LogicalSize } = await import("@tauri-apps/api/dpi");
    await getCurrentWindow().setSize(new LogicalSize(
      s.window_size.width,
      s.window_size.height,
    ));
  } catch {}

  setOnToggleSettings(toggleSettings);
  setSettingsOpenChecker(() => settingsVisible.value);
  window.addEventListener("resize", onResize);

  window.addEventListener("beforeunload", onSaveState);

  unlistenDragDrop = await startDragDrop();

  unlistenCli = await listen<string>("cli-file-open", async (event) => {
    await loadFromPath(event.payload);
  });
});

onUnmounted(() => {
  window.removeEventListener("resize", onResize);
  window.removeEventListener("beforeunload", onSaveState);
  unlistenCli?.();
  unlistenDragDrop?.();
});
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  width: 100%;
  height: 100%;
  overflow: hidden;
  color: #fff;
  font-family: 'Segoe UI', 'Meiryo', sans-serif;
  user-select: none;
}

#app {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
