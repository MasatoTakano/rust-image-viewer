type Segment = { type: "num"; val: number } | { type: "txt"; val: string };

function naturalSortKey(s: string): Segment[] {
  const segments: Segment[] = [];
  let i = 0;
  while (i < s.length) {
    if (s.charCodeAt(i) >= 48 && s.charCodeAt(i) <= 57) {
      let numStr = "";
      while (i < s.length && s.charCodeAt(i) >= 48 && s.charCodeAt(i) <= 57) {
        numStr += s[i];
        i++;
      }
      segments.push({ type: "num", val: parseInt(numStr, 10) || 0 });
    } else {
      segments.push({ type: "txt", val: s[i].toLowerCase() });
      i++;
    }
  }
  return segments;
}

function compareSegments(a: Segment[], b: Segment[]): number {
  const len = Math.min(a.length, b.length);
  for (let i = 0; i < len; i++) {
    const sa = a[i];
    const sb = b[i];
    if (sa.type !== sb.type) {
      return sa.type === "txt" ? -1 : 1;
    }
    if (sa.type === "num") {
      if (sa.val !== (sb as { type: "num"; val: number }).val) {
        return sa.val - (sb as { type: "num"; val: number }).val;
      }
    } else {
      const cmp = sa.val.localeCompare((sb as { type: "txt"; val: string }).val);
      if (cmp !== 0) return cmp;
    }
  }
  return a.length - b.length;
}

export function sortPathsNaturally(paths: string[]): void {
  paths.sort((a, b) => compareSegments(naturalSortKey(a), naturalSortKey(b)));
}

export function sortEntriesByPath(entries: [string, string][]): void {
  entries.sort((a, b) => compareSegments(naturalSortKey(a[0]), naturalSortKey(b[0])));
}
