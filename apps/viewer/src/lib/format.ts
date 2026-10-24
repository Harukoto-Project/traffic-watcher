const DASH = "-";

function pad(value: number, length = 2): string {
  return String(value).padStart(length, "0");
}

export function formatDateTime(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}/${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

export function formatTime(ms: number): string {
  const d = new Date(ms);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

export function formatShortDateTime(ms: number): string {
  const d = new Date(ms);
  return `${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function makeTickFormatter(spanMs: number): (ms: number) => string {
  const dayMs = 86_400_000;
  if (spanMs > 2 * dayMs) {
    return formatShortDateTime;
  }
  if (spanMs > 30 * 60_000) {
    return (ms) => {
      const d = new Date(ms);
      return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
    };
  }
  return formatTime;
}

export function formatNumber(
  value: number | null | undefined,
  digits = 1,
): string {
  if (value === null || value === undefined || Number.isNaN(value)) {
    return DASH;
  }
  return value.toLocaleString("ja-JP", {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
}

export function formatCount(value: number | null | undefined): string {
  if (value === null || value === undefined) {
    return DASH;
  }
  return value.toLocaleString("ja-JP");
}

export function formatMs(value: number | null | undefined, digits = 1): string {
  if (value === null || value === undefined) {
    return DASH;
  }
  return `${formatNumber(value, digits)}ms`;
}

export function formatPercent(
  value: number | null | undefined,
  digits = 1,
): string {
  if (value === null || value === undefined) {
    return DASH;
  }
  return `${formatNumber(value, digits)}%`;
}

const BYTE_UNITS = ["B", "KB", "MB", "GB", "TB"];

export function formatBytes(value: number | null | undefined): string {
  if (value === null || value === undefined) {
    return DASH;
  }
  let size = value;
  let unit = 0;
  while (size >= 1024 && unit < BYTE_UNITS.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return `${formatNumber(size, unit === 0 ? 0 : 1)}${BYTE_UNITS[unit]}`;
}

export function formatBytesPerSec(value: number | null | undefined): string {
  if (value === null || value === undefined) {
    return DASH;
  }
  return `${formatBytes(value)}/s`;
}

export function formatDuration(ms: number): string {
  const totalSeconds = Math.max(0, Math.round(ms / 1000));
  const days = Math.floor(totalSeconds / 86_400);
  const hours = Math.floor((totalSeconds % 86_400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  if (days > 0) {
    return `${days}日${hours}時間`;
  }
  if (hours > 0) {
    return `${hours}時間${minutes}分`;
  }
  if (minutes > 0) {
    return `${minutes}分${seconds}秒`;
  }
  return `${seconds}秒`;
}

export function fileName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}
