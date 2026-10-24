import { Section } from "@/components/common/Section";
import { useDatabase } from "@/hooks/use-database";
import { formatBytes, formatCount, formatDateTime } from "@/lib/format";

export function DatabaseSummaryCard() {
  const { info } = useDatabase();
  if (!info) {
    return null;
  }

  const rows = [
    { label: "イベント", value: formatCount(info.events) },
    { label: "スナップショット", value: formatCount(info.snapshots) },
    { label: "1時間集計", value: formatCount(info.hourly_rows) },
    { label: "ファイルサイズ", value: formatBytes(info.size_bytes) },
    {
      label: "データ期間",
      value:
        info.first_ms !== null && info.last_ms !== null
          ? `${formatDateTime(info.first_ms)} 〜 ${formatDateTime(info.last_ms)}`
          : "-",
    },
  ];

  return (
    <Section title="データベース">
      <dl className="grid grid-cols-2 gap-x-6 gap-y-3 text-sm lg:grid-cols-5">
        {rows.map((row) => (
          <div key={row.label} className="flex min-w-0 flex-col">
            <dt className="text-muted-foreground">{row.label}</dt>
            <dd className="text-foreground font-medium tabular-nums">
              {row.value}
            </dd>
          </div>
        ))}
        <div className="col-span-full flex min-w-0 flex-col">
          <dt className="text-muted-foreground">記録しているソース</dt>
          <dd className="text-foreground font-medium break-words">
            {info.sources.length > 0 ? info.sources.join("、") : "-"}
          </dd>
        </div>
      </dl>
    </Section>
  );
}
