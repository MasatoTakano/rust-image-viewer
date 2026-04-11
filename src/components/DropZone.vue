<template>
  <div
    class="drop-zone"
    :class="{ 'drag-over': isDragOver }"
    @click="openDialog"
  >
    <div class="drop-message">
      <p>フォルダ、ZIP、RAR ファイルをドラッグ＆ドロップしてください</p>
      <p class="hint">クリックでフォルダ選択 / <button class="link-btn" @click.stop="openArchiveDialog">アーカイブ選択</button></p>
      <p class="hint">Esc で設定画面を開く</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import type { ImageEntry } from "../types";

const emit = defineEmits<{
  loaded: [entries: ImageEntry[]];
  notify: [message: string];
}>();

const isDragOver = ref(false);

async function loadFromPath(path: string) {
  try {
    const entries = await invoke<ImageEntry[]>("load_source", { path });
    if (entries.length === 0) {
      emit("notify", "画像ファイルが見つかりませんでした");
      return;
    }
    emit("loaded", entries);
  } catch (error) {
    emit("notify", `読み込みエラー: ${error}`);
  }
}

async function openDialog() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "画像フォルダを選択",
    });
    if (selected) {
      await loadFromPath(selected as string);
    }
  } catch {
    emit("notify", "ファイルダイアログを開けません");
  }
}

async function openArchiveDialog() {
  try {
    const selected = await open({
      multiple: false,
      title: "アーカイブファイルを選択",
      filters: [
        { name: "アーカイブ", extensions: ["zip", "rar", "cbz", "cbr"] },
      ],
    });
    if (selected) {
      await loadFromPath(selected as string);
    }
  } catch {
    emit("notify", "ファイルダイアログを開けません");
  }
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
  e.stopPropagation();
  isDragOver.value = true;
}

function onDragLeave(e: DragEvent) {
  e.preventDefault();
  e.stopPropagation();
  isDragOver.value = false;
}

let unlisten: (() => void) | null = null;

onMounted(async () => {
  document.addEventListener("dragover", onDragOver);
  document.addEventListener("dragleave", onDragLeave);

  unlisten = await listen<{ paths: string[] }>("tauri://drag-drop", async (event) => {
    const paths = event.payload.paths;
    if (paths.length > 0) {
      await loadFromPath(paths[0]);
    }
  });
});

onUnmounted(() => {
  document.removeEventListener("dragover", onDragOver);
  document.removeEventListener("dragleave", onDragLeave);
  unlisten?.();
});
</script>

<style scoped>
.drop-zone {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.drop-message {
  text-align: center;
  color: #888;
  font-size: 1.2em;
  padding: 40px;
  border: 2px dashed #444;
  border-radius: 12px;
}

.drop-message .hint {
  margin-top: 8px;
  font-size: 0.75em;
  color: #666;
}

.drag-over .drop-message {
  color: #fff;
  border-color: #4a9eff;
  background-color: rgba(74, 158, 255, 0.1);
}

.link-btn {
  background: none;
  border: none;
  color: #4a9eff;
  cursor: pointer;
  font-size: inherit;
  padding: 0;
  text-decoration: underline;
}

.link-btn:hover {
  color: #6ab4ff;
}
</style>
