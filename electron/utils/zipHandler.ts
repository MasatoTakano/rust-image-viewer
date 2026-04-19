import * as fs from "fs";
import * as path from "path";
import JSZip from "jszip";
import { sortEntriesByPath } from "./naturalSort";

const SUPPORTED_EXT = new Set(["jpg", "jpeg", "png", "gif", "bmp", "webp", "avif"]);

function isSupportedImage(filePath: string): boolean {
  const ext = path.extname(filePath).toLowerCase().slice(1);
  return SUPPORTED_EXT.has(ext);
}

export async function enumerateImages(
  archivePath: string,
): Promise<Array<[string, string]>> {
  const data = fs.readFileSync(archivePath);
  const zip = await JSZip.loadAsync(data);

  const entries: Array<[string, string]> = [];

  zip.forEach((relativePath, file) => {
    if (file.dir) return;
    if (isSupportedImage(relativePath)) {
      entries.push([relativePath, path.basename(relativePath)]);
    }
  });

  sortEntriesByPath(entries);
  return entries;
}

export async function readImageData(
  archivePath: string,
  entryPath: string,
): Promise<Buffer> {
  const data = fs.readFileSync(archivePath);
  const zip = await JSZip.loadAsync(data);
  const file = zip.file(entryPath);
  if (!file) throw new Error(`エントリが見つかりません: ${entryPath}`);
  return file.async("nodebuffer");
}
