import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import type { ChangeEventHandler } from "react";

type FilePickerProps = {
  disabled: boolean;
  onChange: ChangeEventHandler<HTMLInputElement>;
};

export function FilePicker({ disabled, onChange }: FilePickerProps) {
  return (
    <div className="space-y-2">
      <Label htmlFor="file">Choose images</Label>
      <Input id="file" type="file" accept="image/*" multiple disabled={disabled} onChange={onChange} />
    </div>
  );
}
