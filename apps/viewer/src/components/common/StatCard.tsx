import { Card, CardContent } from "@/components/ui/card";

interface StatCardProps {
  label: string;
  value: string;
  hint?: string;
}

export function StatCard({ label, value, hint }: StatCardProps) {
  return (
    <Card size="sm" className="min-w-0">
      <CardContent className="flex flex-col gap-1">
        <p className="text-muted-foreground text-sm">{label}</p>
        <p className="text-foreground text-2xl font-semibold tabular-nums">
          {value}
        </p>
        <p className="text-muted-foreground min-h-5 text-xs tabular-nums">
          {hint}
        </p>
      </CardContent>
    </Card>
  );
}
