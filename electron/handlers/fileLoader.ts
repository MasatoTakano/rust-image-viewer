import * as fs from "fs";
import * as path from "path";
import { sortPathsNaturally } from "../utils/naturalSort";

const SUPPORTED_EXT = new Set(["jpg", "jpeg", "png", "gif", "bmp", "webp", "avif"]);

function isSupportedImage(filePath: string): boolean {
  const ext = path.extname(filePath).toLowerCase().slice(1);
  return SUPPORTED_EXT.has(ext);
}

function walkDir(dir: string, callback: (filePath: string) => void): void {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walkDir(fullPath, callback);
    } else if (entry.isFile()) {
      callback(fullPath);
    }
  }
}

interface ImageEntry {
  index: number;
  display_name: string;
  source: any;
}

export async function loadSource(inputPath: string): Promise<ImageEntry[]> {
  if (!fs.existsSync(inputPath)) {
    throw new Error(`パスが存在しません: ${inputPath}`);
  }

  const stat = fs.statSync(inputPath);

  if (stat.isDirectory()) {
    return enumerateFolder(inputPath);
  }

  const ext = path.extname(inputPath).toLowerCase().slice(1);
  switch (ext) {
    case "zip":
      return enumerateZip(inputPath);
    case "rar":
      throw new Error("RAR形式は現バージョンではサポートされていません。ZIPまたはフォルダを使用してください。");
    default:
      throw new Error("対応していないファイル形式です。フォルダ、ZIPをサポートしています。");
  }
}

function enumerateFolder(folderPath: string): ImageEntry[] {
  const paths: string[] = [];
  walkDir(folderPath, (filePath) => {
    if (isSupportedImage(filePath)) {
      paths.push(filePath);
    }
  });

  sortPathsNaturally(paths);

  return paths.map((p, index) => ({
    index,
    display_name: path.basename(p),
    source: { FileSystem: { path: p } },
  }));
}

async function enumerateZip(archivePath: string): Promise<ImageEntry[]> {
  const { enumerateImages } = await import("../utils/zipHandler");
  const rawEntries = await enumerateImages(archivePath);

  return rawEntries.map(([entryPath, displayName], index) => ({
    index,
    display_name: displayName,
    source: { Zip: { archive_path: archivePath, entry_path: entryPath } },
  }));
}
