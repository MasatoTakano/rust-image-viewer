<template>
  <Teleport to="body">
    <div v-if="visible" class="settings-overlay" @click.self="close">
      <div class="settings-content" @keydown.stop>
        <h2>設定</h2>
        <h3 class="section-title">キーバインド</h3>
        <table>
          <tr v-for="(label, key) in bindingLabels" :key="key">
            <td>{{ label }}</td>
            <td>
              <button
                class="key-binding-btn"
                :class="{ recording: recordingKey === key }"
                @click="startRecording(key as keyof KeyBindings)"
                @keydown.prevent="onKeyRecord"
                tabindex="0"
              >
                <template v-if="recordingKey === key">キーを押してください...</template>
                <template v-else-if="bindings[key as keyof KeyBindings]?.length">
                  {{ formatKeyCombo(bindings[key as keyof KeyBindings]![0]) }}
                </template>
                <template v-else>クリックして設定</template>
              </button>
            </td>
          </tr>
        </table>
        <h3 class="section-title">表示</h3>
        <table>
          <tr>
            <td>背景色</td>
            <td>
              <input type="color" v-model="bgColor" />
            </td>
          </tr>
        </table>
        <h3 class="section-title">動作</h3>
        <table>
          <tr>
            <td>先読み枚数</td>
            <td>
              <input type="number" v-model.number="preloadRange" min="0" max="50" />
            </td>
          </tr>
          <tr>
            <td>キー反応間隔 (ms)</td>
            <td>
              <input type="number" v-model.number="keyThrottleMs" min="0" max="1000" />
            </td>
          </tr>
          <tr>
            <td>ホイール反応間隔 (ms)</td>
            <td>
              <input type="number" v-model.number="wheelThrottleMs" min="0" max="1000" />
            </td>
          </tr>
        </table>
        <div class="settings-actions">
          <button @click="close">閉じる</button>
          <button class="primary" @click="handleSave">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import type { KeyBindings, AppSettings } from "../types";
import { useSettings } from "../composables/useSettings";
import { buildKeyCombo, formatKeyCombo } from "../utils/keyCombo";

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  close: [];
  saved: [];
  notify: [message: string];
}>();

const { settings, save } = useSettings();

const bindingLabels: Record<keyof KeyBindings, string> = {
  next_page: "次ページ",
  prev_page: "前ページ",
  toggle_fullscreen: "フルスクリーン切替",
  toggle_spread: "表示モード切替",
  go_first: "最初のページ",
  go_last: "最後のページ",
  open_settings: "設定を開く",
};

const bindings = ref<KeyBindings>({ ...settings.value.key_bindings });
const bgColor = ref(settings.value.background_color);
const preloadRange = ref(settings.value.preload_range);
const keyThrottleMs = ref(settings.value.key_throttle_ms);
const wheelThrottleMs = ref(settings.value.wheel_throttle_ms);

const recordingKey = ref<keyof KeyBindings | null>(null);

watch(
  () => props.visible,
  (v) => {
    if (v) {
      bindings.value = { ...settings.value.key_bindings };
      bgColor.value = settings.value.background_color;
      preloadRange.value = settings.value.preload_range;
      keyThrottleMs.value = settings.value.key_throttle_ms;
      wheelThrottleMs.value = settings.value.wheel_throttle_ms;
      recordingKey.value = null;
    }
  }
);

function startRecording(key: keyof KeyBindings) {
  recordingKey.value = key;
}

function onKeyRecord(e: KeyboardEvent) {
  if (!recordingKey.value) return;
  if (e.code === "Escape") {
    recordingKey.value = null;
    return;
  }
  const combo = buildKeyCombo(e);
  bindings.value[recordingKey.value] = [combo];
  recordingKey.value = null;
}

function close() {
  recordingKey.value = null;
  emit("close");
}

async function handleSave() {
  const newSettings: AppSettings = {
    ...settings.value,
    key_bindings: { ...bindings.value },
    background_color: bgColor.value,
    preload_range: preloadRange.value,
    key_throttle_ms: keyThrottleMs.value,
    wheel_throttle_ms: wheelThrottleMs.value,
  };
  try {
    await save(newSettings);
    emit("saved");
    emit("close");
    emit("notify", "設定を保存しました");
  } catch (error) {
    emit("notify", `設定の保存に失敗: ${error}`);
  }
}
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-content {
  background-color: #1e1e1e;
  border-radius: 8px;
  padding: 24px;
  min-width: 400px;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.settings-content h2 {
  margin-bottom: 16px;
  color: #fff;
}

.section-title {
  margin-top: 16px;
  margin-bottom: 8px;
  color: #aaa;
  font-size: 0.85em;
  font-weight: normal;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.settings-content table {
  width: 100%;
  border-collapse: collapse;
}

.settings-content td {
  padding: 6px 8px;
  border-bottom: 1px solid #333;
  vertical-align: middle;
}

.settings-content td:first-child {
  color: #ccc;
  width: 180px;
}

.settings-content input {
  background-color: #2d2d2d;
  border: 1px solid #555;
  color: #fff;
  padding: 4px 8px;
  border-radius: 4px;
  width: 100%;
  font-size: 0.9em;
}

.settings-content input:focus {
  outline: none;
  border-color: #4a9eff;
}

.key-binding-btn {
  background-color: #2d2d2d;
  border: 1px solid #555;
  color: #ccc;
  padding: 6px 12px;
  border-radius: 4px;
  width: 100%;
  text-align: center;
  cursor: pointer;
  font-size: 0.9em;
  font-family: inherit;
  transition: border-color 0.15s, color 0.15s;
}

.key-binding-btn:hover {
  border-color: #777;
  color: #fff;
}

.key-binding-btn.recording {
  border-color: #4a9eff;
  color: #4a9eff;
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.settings-actions {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.settings-actions button {
  padding: 6px 16px;
  border: 1px solid #555;
  border-radius: 4px;
  background-color: #2d2d2d;
  color: #fff;
  cursor: pointer;
  font-size: 0.9em;
}

.settings-actions button:hover {
  background-color: #3d3d3d;
}

.settings-actions button.primary {
  background-color: #4a9eff;
  border-color: #4a9eff;
}

.settings-actions button.primary:hover {
  background-color: #3a8eef;
}
</style>
