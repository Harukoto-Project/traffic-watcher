import { Input } from "@/components/ui/input";
import { useTimeRange } from "@/hooks/use-time-range";
import { fromInputValue, toInputValue } from "@/lib/dateInput";

export function CustomRangeInputs() {
  const { range, setCustom } = useTimeRange();

  const handleSince = (value: string) => {
    const sinceMs = fromInputValue(value);
    if (sinceMs !== null && sinceMs < range.untilMs) {
      setCustom(sinceMs, range.untilMs);
    }
  };

  const handleUntil = (value: string) => {
    const untilMs = fromInputValue(value);
    if (untilMs !== null && untilMs > range.sinceMs) {
      setCustom(range.sinceMs, untilMs);
    }
  };

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Input
        type="datetime-local"
        step={1}
        className="w-56"
        aria-label="開始時刻"
        value={toInputValue(range.sinceMs)}
        onChange={(event) => handleSince(event.target.value)}
      />
      <span className="text-muted-foreground text-xs">〜</span>
      <Input
        type="datetime-local"
        step={1}
        className="w-56"
        aria-label="終了時刻"
        value={toInputValue(range.untilMs)}
        onChange={(event) => handleUntil(event.target.value)}
      />
    </div>
  );
}
