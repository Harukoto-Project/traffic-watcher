import { StatCard } from "@/components/common/StatCard";
import { formatBytesPerSec, formatCount, formatPercent } from "@/lib/format";
import type { SystemSummary } from "@/types/monitor";

interface SystemSummaryGridProps {
  system: SystemSummary;
  alertCount: number;
}

export function SystemSummaryGrid({
  system,
  alertCount,
}: SystemSummaryGridProps) {
  return (
    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
      <StatCard label="アラート" value={`${formatCount(alertCount)}件`} />
      <StatCard
        label="CPU平均"
        value={formatPercent(system.cpu_avg)}
        hint={`最大${formatPercent(system.cpu_max)}`}
      />
      <StatCard
        label="メモリ平均"
        value={formatPercent(system.mem_avg)}
        hint={`最大${formatPercent(system.mem_max)}`}
      />
      <StatCard
        label="システム採取数"
        value={`${formatCount(system.samples)}回`}
      />
      <StatCard
        label="受信平均"
        value={formatBytesPerSec(system.rx_avg)}
        hint={`最大${formatBytesPerSec(system.rx_max)}`}
      />
      <StatCard
        label="送信平均"
        value={formatBytesPerSec(system.tx_avg)}
        hint={`最大${formatBytesPerSec(system.tx_max)}`}
      />
    </div>
  );
}
