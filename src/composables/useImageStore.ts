import { ref, readonly } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ImageEntry, ImageResult } from "../types";
import { useSettings } from "./useSettings";
import { useFullscreen } from "./useFullscreen";

const PAGE_SLIDER_HEIGHT = 28;
const STATUS_BAR_HEIGHT = 28;
const UI_OVERHEAD_HEIGHT = PAGE_SLIDER_HEIGHT + STATUS_BAR_HEIGHT;
const MAX_DEVICE_PIXEL_RATIO = 2;
const MAX_CACHE_SIZE = 30;

const entries = ref<ImageEntry[]>([]);
const cache = new Map<number, string>();
const pendingFetches = new Map<number, Promise<string | null>>();
let cacheGeneration = 0;

export function useImageStore() {
  const { settings } = useSettings();
  const { isFullscreen } = useFullscreen();

  function setEntries(list: ImageEntry[]) {
    entries.value = list;
    cache.clear();
    cacheGeneration++;
  }

  function trimCache(centerIndex: number) {
    if (cache.size <= MAX_CACHE_SIZE) return;
    const keys = Array.from(cache.keys());
    keys.sort((a, b) => Math.abs(b - centerIndex) - Math.abs(a - centerIndex));
    for (let i = 0; cache.size > MAX_CACHE_SIZE && i < keys.length; i++) {
      cache.delete(keys[i]);
    }
  }

  function getMaxViewport(): { maxWidth: number; maxHeight: number } {
    const dpr = Math.min(window.devicePixelRatio || 1, MAX_DEVICE_PIXEL_RATIO);
    const overhead = isFullscreen.value ? 0 : UI_OVERHEAD_HEIGHT;
    return {
      maxWidth: Math.floor(window.innerWidth * dpr),
      maxHeight: Math.floor((window.innerHeight - overhead) * dpr),
    };
  }

  async function getImage(index: number): Promise<string | null> {
    if (index < 0 || index >= entries.value.length) return null;

    const cached = cache.get(index);
    if (cached) return cached;

    const result = await fetchAndCache(index);
    trimCache(index);
    return result;
  }

  async function fetchAndCache(index: number): Promise<string | null> {
    const pending = pendingFetches.get(index);
    if (pending) return pending;

    const entry = entries.value[index];
    const gen = cacheGeneration;
    const { maxWidth, maxHeight } = getMaxViewport();
    const promise = (async () => {
      try {
        const result = await invoke<ImageResult>("get_image", {
          entry,
          maxWidth,
          maxHeight,
        });
        if (gen === cacheGeneration) {
          cache.set(index, result.data_url);
        }
        return result.data_url;
      } catch (error) {
        console.error(`画像の読み込みに失敗 (index: ${index}):`, error);
        return null;
      } finally {
        pendingFetches.delete(index);
      }
    })();
    pendingFetches.set(index, promise);
    return promise;
  }

  function getRangeBounds(index: number): [number, number] {
    const range = settings.value.preload_range;
    const start = Math.max(0, index - range);
    const end = Math.min(entries.value.length - 1, index + range);
    return [start, end];
  }

  function preloadAround(index: number) {
    const range = settings.value.preload_range;
    const len = entries.value.length;

    for (let offset = 1; offset <= range; offset++) {
      const i = index + offset;
      if (i < len && !cache.has(i) && !pendingFetches.has(i)) {
        fetchAndCache(i).catch(() => {});
      }
    }
    for (let offset = 1; offset <= range; offset++) {
      const i = index - offset;
      if (i >= 0 && !cache.has(i) && !pendingFetches.has(i)) {
        fetchAndCache(i).catch(() => {});
      }
    }

    trimCache(index);
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
