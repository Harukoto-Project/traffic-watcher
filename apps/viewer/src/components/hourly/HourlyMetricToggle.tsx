import { hourlyMetrics } from "@/config/hourlyMetrics";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

interface HourlyMetricToggleProps {
  value: string;
  onChange: (id: string) => void;
}

export function HourlyMetricToggle({
  value,
  onChange,
}: HourlyMetricToggleProps) {
  return (
    <div className="max-w-full overflow-x-auto">
      <ToggleGroup
        type="single"
        variant="outline"
        size="sm"
        spacing={0}
        value={value}
        onValueChange={(next) => {
          if (next) {
            onChange(next);
          }
        }}
        aria-label="指標"
      >
        {hourlyMetrics.map((metric) => (
          <ToggleGroupItem key={metric.id} value={metric.id}>
            {metric.label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}
