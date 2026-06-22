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
import { open } from "@tauri-apps/plugin-dialog";

const emit = defineEmits<{
  "path-selected": [path: string];
  notify: [message: string];
}>();

const isDragOver = ref(false);

async function openDialog() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "画像フォルダを選択",
    });
    if (selected) {
      emit("path-selected", selected as string);
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
      emit("path-selected", selected as string);
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

onMounted(() => {
  document.addEventListener("dragover", onDragOver);
  document.addEventListener("dragleave", onDragLeave);
});

onUnmounted(() => {
  document.removeEventListener("dragover", onDragOver);
  document.removeEventListener("dragleave", onDragLeave);
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
