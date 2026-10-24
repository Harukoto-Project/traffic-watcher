import {
  Area,
  AreaChart,
  CartesianGrid,
  Line,
  LineChart,
  ReferenceLine,
  XAxis,
  YAxis,
} from "recharts";

import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";
import { STATUS_STYLES } from "@/config/status";
import type { ChartRow, ChartSeries } from "@/lib/chartData";
import { formatDateTime, makeTickFormatter } from "@/lib/format";

import { SeriesLegend } from "./SeriesLegend";

interface TimeSeriesChartProps {
  rows: ChartRow[];
  series: ChartSeries[];
  sinceMs: number;
  untilMs: number;
  variant?: "line" | "area";
  markers?: number[];
  syncId?: string;
  connectNulls?: boolean;
  stepped?: boolean;
  yMax?: number | null;
  yFormatter?: (value: number) => string;
  height?: number;
}

const MARGIN = { top: 8, right: 12, bottom: 0, left: 0 };
const Y_AXIS_WIDTH = 56;

function buildConfig(series: ChartSeries[]): ChartConfig {
  const config: ChartConfig = {};
  for (const item of series) {
    config[item.key] = { label: item.label, color: item.color };
  }
  return config;
}

export function TimeSeriesChart({
  rows,
  series,
  sinceMs,
  untilMs,
  variant = "line",
  markers = [],
  syncId,
  connectNulls = false,
  stepped = false,
  yMax = null,
  yFormatter,
  height = 224,
}: TimeSeriesChartProps) {
  const config = buildConfig(series);
  const curve = stepped ? "stepAfter" : "linear";

  const frame = [
    <CartesianGrid key="grid" vertical={false} />,
    <XAxis
      key="x"
      dataKey="ts"
      type="number"
      scale="time"
      domain={[sinceMs, untilMs]}
      allowDataOverflow
      tickCount={6}
      tickLine={false}
      axisLine={false}
      tickMargin={8}
      tickFormatter={makeTickFormatter(untilMs - sinceMs)}
    />,
    <YAxis
      key="y"
      width={Y_AXIS_WIDTH}
      domain={[0, yMax ?? "auto"]}
      tickLine={false}
      axisLine={false}
      tickFormatter={yFormatter}
    />,
    <ChartTooltip
      key="tooltip"
      content={
        <ChartTooltipContent
          labelFormatter={(_label, payload) => {
            const ts = payload?.[0]?.payload?.ts;
            return typeof ts === "number" ? formatDateTime(ts) : "";
          }}
        />
      }
    />,
    ...markers.map((ts) => (
      <ReferenceLine
        key={`marker-${ts}`}
        x={ts}
        stroke={STATUS_STYLES.critical.color}
        strokeDasharray="3 3"
        ifOverflow="hidden"
      />
    )),
  ];

  return (
    <div className="flex flex-col gap-3">
      {series.length > 1 ? <SeriesLegend series={series} /> : null}
      <ChartContainer
        config={config}
        className="aspect-auto w-full"
        style={{ height }}
      >
        {variant === "area" ? (
          <AreaChart data={rows} margin={MARGIN} syncId={syncId}>
            {frame}
            {series.map((item) => (
              <Area
                key={item.key}
                dataKey={item.key}
                type={curve}
                stroke={`var(--color-${item.key})`}
                fill={`var(--color-${item.key})`}
                fillOpacity={0.25}
                strokeWidth={1.5}
                strokeDasharray={item.dash}
                dot={false}
                isAnimationActive={false}
                connectNulls={connectNulls}
              />
            ))}
          </AreaChart>
        ) : (
          <LineChart data={rows} margin={MARGIN} syncId={syncId}>
            {frame}
            {series.map((item) => (
              <Line
                key={item.key}
                dataKey={item.key}
                type={curve}
                stroke={`var(--color-${item.key})`}
                strokeWidth={1.5}
                strokeDasharray={item.dash}
                dot={false}
                isAnimationActive={false}
                connectNulls={connectNulls}
              />
            ))}
          </LineChart>
        )}
      </ChartContainer>
    </div>
  );
}
