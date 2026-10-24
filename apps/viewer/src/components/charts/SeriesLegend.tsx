import type { ChartSeries } from "@/lib/chartData";

function LineSample({ series }: { series: ChartSeries }) {
  return (
    <svg width="24" height="8" aria-hidden="true" className="shrink-0">
      <line
        x1="0"
        y1="4"
        x2="24"
        y2="4"
        stroke={series.color}
        strokeWidth="2"
        strokeDasharray={series.dash}
      />
    </svg>
  );
}

export function SeriesLegend({ series }: { series: ChartSeries[] }) {
  return (
    <ul className="text-muted-foreground flex flex-wrap gap-x-4 gap-y-1 text-xs">
      {series.map((item) => (
        <li key={item.key} className="flex items-center gap-2">
          <LineSample series={item} />
          {item.label}
        </li>
      ))}
    </ul>
  );
}
