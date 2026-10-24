export type HourlySourceKind = "ping" | "system";

export type HourlyTransform = "none" | "lossPct" | "megabytes";

export interface HourlyMetric {
  id: string;
  label: string;
  sourceKind: HourlySourceKind;
  metric: string;
  unit: string;
  transform: HourlyTransform;
  showMax: boolean;
  fixedMax: number | null;
}

export const hourlyMetrics: HourlyMetric[] = [
  {
    id: "rtt",
    label: "pingの往復時間",
    sourceKind: "ping",
    metric: "rtt_ms",
    unit: "ms",
    transform: "none",
    showMax: true,
    fixedMax: null,
  },
  {
    id: "loss",
    label: "pingのロス率",
    sourceKind: "ping",
    metric: "ok",
    unit: "%",
    transform: "lossPct",
    showMax: false,
    fixedMax: 100,
  },
  {
    id: "cpu",
    label: "CPU使用率",
    sourceKind: "system",
    metric: "cpu_pct",
    unit: "%",
    transform: "none",
    showMax: true,
    fixedMax: 100,
  },
  {
    id: "memory",
    label: "メモリ使用率",
    sourceKind: "system",
    metric: "mem_used_pct",
    unit: "%",
    transform: "none",
    showMax: true,
    fixedMax: 100,
  },
  {
    id: "rx",
    label: "ネットワーク受信",
    sourceKind: "system",
    metric: "rx_bytes_s",
    unit: "MB/s",
    transform: "megabytes",
    showMax: true,
    fixedMax: null,
  },
  {
    id: "tx",
    label: "ネットワーク送信",
    sourceKind: "system",
    metric: "tx_bytes_s",
    unit: "MB/s",
    transform: "megabytes",
    showMax: true,
    fixedMax: null,
  },
];

export const DEFAULT_HOURLY_METRIC_ID = "rtt";
