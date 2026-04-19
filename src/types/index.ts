export interface ImageEntry {
  index: number;
  display_name: string;
  source: ImageSource;
}

export type ImageSource =
  | { FileSystem: { path: string } }
  | { Zip: { archive_path: string; entry_path: string } }
  | { Rar: { archive_path: string; entry_path: string } };

export interface ImageResult {
  data_url: string;
}

export interface AppSettings {
  key_bindings: KeyBindings;
  window_size: WindowSize;
  background_color: string;
  preload_range: number;
  key_throttle_ms: number;
  wheel_throttle_ms: number;
  display_mode: string;
  resize_filter: string;
}

export interface KeyBindings {
  next_page: readonly string[];
  prev_page: readonly string[];
  toggle_fullscreen: readonly string[];
  toggle_spread: readonly string[];
  go_first: readonly string[];
  go_last: readonly string[];
  open_settings: readonly string[];
}

export interface WindowSize {
  width: number;
  height: number;
}

export enum DisplayMode {
  Single = "single",
  Spread = "spread",
}

export interface ElectronAPI {
  loadSource(p: string): Promise<ImageEntry[]>;
  getImage(entry: ImageEntry, maxWidth: number, maxHeight: number, filterType?: string): Promise<ImageResult>;
  loadSettings(): Promise<AppSettings>;
  saveSettings(settings: AppSettings): Promise<void>;
  openDialog(options: {
    directory?: boolean;
    multiple?: boolean;
    title?: string;
    filters?: Array<{ name: string; extensions: string[] }>;
  }): Promise<string | null>;
  setFullscreen(fullscreen: boolean): Promise<void>;
  isFullscreen(): Promise<boolean>;
  setWindowSize(width: number, height: number): Promise<void>;
  onCliFileOpen(callback: (p: string) => void): () => void;
  onFileDropped: ((p: string) => void) | null;
  onDragStateChanged: ((over: boolean) => void) | null;
  getFilePath(file: File): string;
}
