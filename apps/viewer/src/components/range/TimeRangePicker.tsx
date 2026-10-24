import { rangePresets } from "@/config/timeRangePresets";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useTimeRange } from "@/hooks/use-time-range";
import { formatDateTime } from "@/lib/format";
import type { RangePresetId } from "@/types/timeRange";

import { CustomRangeInputs } from "./CustomRangeInputs";

export function TimeRangePicker() {
  const { selection, range, setPreset } = useTimeRange();

  return (
    <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
      <div className="max-w-full overflow-x-auto">
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          spacing={0}
          value={selection.preset}
          onValueChange={(value) => {
            if (value) {
              setPreset(value as RangePresetId);
            }
          }}
          aria-label="表示期間"
        >
          {rangePresets.map((preset) => (
            <ToggleGroupItem key={preset.id} value={preset.id}>
              {preset.label}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>
      {selection.preset === "custom" ? (
        <CustomRangeInputs />
      ) : (
        <p className="text-muted-foreground text-sm tabular-nums">
          {formatDateTime(range.sinceMs)} 〜 {formatDateTime(range.untilMs)}
        </p>
      )}
    </div>
  );
}
