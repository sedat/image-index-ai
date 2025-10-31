import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

type SearchBarProps = {
  defaultQuery?: string;
};

export function SearchBar({ defaultQuery }: SearchBarProps) {
  return (
    <form action="/" method="get" className="flex items-center gap-2">
      <Input name="q" placeholder="Search by description or tags..." defaultValue={defaultQuery ?? ""} />
      <Button type="submit">Search</Button>
    </form>
  );
}
