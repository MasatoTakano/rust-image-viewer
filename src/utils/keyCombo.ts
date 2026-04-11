export function buildKeyCombo(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey && e.code !== "ControlLeft" && e.code !== "ControlRight") parts.push("Control");
  if (e.altKey && e.code !== "AltLeft" && e.code !== "AltRight") parts.push("Alt");
  if (e.shiftKey && e.code !== "ShiftLeft" && e.code !== "ShiftRight") parts.push("Shift");

  if (["Space", "Backspace", "Home", "End", "F11", "Enter"].includes(e.code) || e.code.startsWith("Arrow")) {
    parts.push(e.code);
  } else if (e.code === "Comma" || e.code.startsWith("Key")) {
    parts.push(e.code);
  } else {
    parts.push(e.key);
  }
  return parts.join("+");
}

const KEY_DISPLAY: Record<string, string> = {
  ArrowLeft: "\u2190",
  ArrowRight: "\u2192",
  ArrowUp: "\u2191",
  ArrowDown: "\u2193",
  Space: "Space",
  Enter: "Enter",
  Backspace: "Backspace",
  Home: "Home",
  End: "End",
};

export function formatKeyCombo(combo: string): string {
  return combo.split("+").map((part) => {
    if (KEY_DISPLAY[part]) return KEY_DISPLAY[part];
    if (part === "Control") return "Ctrl";
    if (part.startsWith("Key")) return part.slice(3);
    if (part.startsWith("Digit")) return part.slice(5);
    return part;
  }).join(" + ");
}
