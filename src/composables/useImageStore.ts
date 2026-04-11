import { ref, readonly } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ImageEntry, ImageResult } from "../types";
import { useSettings } from "./useSettings";

const PAGE_SLIDER_HEIGHT = 28;
const STATUS_BAR_HEIGHT = 28;
const UI_OVERHEAD_HEIGHT = PAGE_SLIDER_HEIGHT + STATUS_BAR_HEIGHT;

const entries = ref<ImageEntry[]>([]);
const cache = new Map<number, string>();
let cacheGeneration = 0;

export function useImageStore() {
  const { settings } = useSettings();

  function setEntries(list: ImageEntry[]) {
    entries.value = list;
    cache.clear();
    cacheGeneration++;
  }

  async function getImage(index: number): Promise<string | null> {
    if (index < 0 || index >= entries.value.length) return null;

    const cached = cache.get(index);
    if (cached) return cached;

    return fetchAndCache(index);
  }

  async function fetchAndCache(index: number): Promise<string | null> {
    const entry = entries.value[index];
    const gen = cacheGeneration;
    try {
      const result = await invoke<ImageResult>("get_image", {
        entry,
        maxWidth: window.innerWidth,
        maxHeight: window.innerHeight - UI_OVERHEAD_HEIGHT,
      });
      if (gen === cacheGeneration) {
        cache.set(index, result.data_url);
      }
      return result.data_url;
    } catch (error) {
      console.error(`画像の読み込みに失敗 (index: ${index}):`, error);
      return null;
    }
  }

  function getRangeBounds(index: number): [number, number] {
    const range = settings.value.preload_range;
    const start = Math.max(0, index - range);
    const end = Math.min(entries.value.length - 1, index + range);
    return [start, end];
  }

  function preloadAround(index: number) {
    const [start, end] = getRangeBounds(index);
    for (let i = start; i <= end; i++) {
      if (!cache.has(i)) {
        fetchAndCache(i).catch(() => {});
      }
    }
  }

  function refreshInBackground(index: number) {
    cacheGeneration++;
    const [start, end] = getRangeBounds(index);
    for (let i = start; i <= end; i++) {
      fetchAndCache(i).catch(() => {});
    }
  }

  return {
    entries: readonly(entries),
    setEntries,
    getImage,
    preloadAround,
    refreshInBackground,
    cacheHas: (index: number) => cache.has(index),
  };
}
