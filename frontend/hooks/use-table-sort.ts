import { useCallback, useState } from "react";

type TableSort<THeaderId extends string> = {
  id: THeaderId;
  reverse: boolean;
};

export function useTableSort<THeaderId extends string>(defaultId: THeaderId) {
  const [sort, setSort] = useState<TableSort<THeaderId>>({
    id: defaultId,
    reverse: false,
  });

  const updateSort = useCallback((id: THeaderId) => {
    setSort((previousSort) => ({
      id,
      reverse: previousSort?.id === id && !previousSort.reverse,
    }));
  }, []);

  return [sort, updateSort] as const;
}
