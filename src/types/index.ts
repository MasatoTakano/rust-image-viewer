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
}

export interface KeyBindings {
  next_page: string[];
  prev_page: string[];
  toggle_fullscreen: string[];
  toggle_spread: string[];
  go_first: string[];
  go_last: string[];
  open_settings: string[];
}

export interface WindowSize {
  width: number;
  height: number;
}

export enum DisplayMode {
  Single = "single",
  Spread = "spread",
}
