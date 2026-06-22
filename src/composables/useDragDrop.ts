import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ImageEntry } from "../types";

export function useDragDrop(
  onLoaded: (entries: ImageEntry[]) => void,
  onNotify: (message: string) => void,
) {
  async function loadFromPath(path: string) {
    try {
      const entries = await invoke<ImageEntry[]>("load_source", { path });
      if (entries.length === 0) {
        onNotify("画像ファイルが見つかりませんでした");
        return;
      }
      onLoaded(entries);
    } catch (error) {
      onNotify(`読み込みエラー: ${error}`);
    }
  }

  function preventDefault(e: DragEvent) {
    e.preventDefault();
  }

  async function start(): Promise<() => void> {
    document.addEventListener("dragover", preventDefault);
    document.addEventListener("drop", preventDefault);

    const unlisten = await listen<{ paths: string[] }>(
      "tauri://drag-drop",
      async (event) => {
        const paths = event.payload.paths;
        if (paths.length > 0) {
          await loadFromPath(paths[0]);
        }
      },
    );

    return () => {
      document.removeEventListener("dragover", preventDefault);
      document.removeEventListener("drop", preventDefault);
      unlisten();
    };
  }

  return { start, loadFromPath };
}
