export class AppError extends Error {
  code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = "AppError";
    this.code = code;
  }
}

function isCommandError(
  value: unknown,
): value is { code: string; message: string } {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const record = value as Record<string, unknown>;
  return typeof record.code === "string" && typeof record.message === "string";
}

export function toAppError(err: unknown): AppError {
  if (err instanceof AppError) {
    return err;
  }
  if (isCommandError(err)) {
    return new AppError(err.code, err.message);
  }
  if (err instanceof Error) {
    return new AppError("unknown", err.message);
  }
  return new AppError("unknown", String(err));
}
