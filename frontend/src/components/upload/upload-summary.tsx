type UploadSummaryProps = {
  total: number;
  success: number;
  failed: number;
  inProgress: number;
};

export function UploadSummary({ total, success, failed, inProgress }: UploadSummaryProps) {
  return (
    <div className="text-sm">
      <p className="font-medium">Summary</p>
      <p className="text-muted-foreground">
        Total: {total} · Uploading: {inProgress} · Success: {success} · Failed: {failed}
      </p>
    </div>
  );
}
