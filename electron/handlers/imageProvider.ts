import * as fs from "fs";
import * as path from "path";
import sharp from "sharp";

interface ImageEntry {
  source: any;
  display_name: string;
}

const KERNEL_MAP: Record<string, string> = {
  nearest: "nearest",
  triangle: "cubic",
  catmull_rom: "cubic",
  gaussian: "lanczos3",
  lanczos3: "lanczos3",
};

async function readRawData(entry: ImageEntry): Promise<Buffer> {
  if ("FileSystem" in entry.source) {
    return fs.readFileSync(entry.source.FileSystem.path);
  }
  if ("Zip" in entry.source) {
    const { readImageData } = await import("../utils/zipHandler");
    return readImageData(entry.source.Zip.archive_path, entry.source.Zip.entry_path);
  }
  if ("Rar" in entry.source) {
    throw new Error("RAR support is not yet available");
  }
  throw new Error("Unknown source type");
}

export async function getImage(
  entry: ImageEntry,
  maxWidth: number,
  maxHeight: number,
  filterType?: string,
): Promise<{ data_url: string }> {
  const raw = await readRawData(entry);
  const ext = path.extname(entry.display_name).toLowerCase().slice(1);

  if (["avif", "webp", "gif"].includes(ext)) {
    const mime = `image/${ext}`;
    return { data_url: `data:${mime};base64,${raw.toString("base64")}` };
  }

  const metadata = await sharp(raw).metadata();
  const width = metadata.width || 0;
  const height = metadata.height || 0;

  if (width <= maxWidth && height <= maxHeight) {
    if (ext === "jpg" || ext === "jpeg") {
      return { data_url: `data:image/jpeg;base64,${raw.toString("base64")}` };
    }
    if (ext === "png") {
      return { data_url: `data:image/png;base64,${raw.toString("base64")}` };
    }
  }

  const needsResize = width > maxWidth || height > maxHeight;

  if (needsResize) {
    const kernel = KERNEL_MAP[filterType ?? "catmull_rom"] ?? "cubic";
    const buf = await sharp(raw)
      .resize(maxWidth, maxHeight, {
        fit: "inside",
        kernel: kernel as any,
      })
      .jpeg({ quality: 92 })
      .toBuffer();
    return { data_url: `data:image/jpeg;base64,${buf.toString("base64")}` };
  }

  const buf = await sharp(raw).png().toBuffer();
  return { data_url: `data:image/png;base64,${buf.toString("base64")}` };
}
