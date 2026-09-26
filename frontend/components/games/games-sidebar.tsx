import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { useDataQuery } from "@hooks/use-data-query";
import { GamesSortMenu } from "./games-sort-menu";
import { FilterList } from "./filters/filter-list";
import { Stack } from "@mantine/core";

export function GamesSidebar() {
	const [dataQuery, setDataQuery] = useDataQuery();

	return (
		<>
			<Stack p="xs">
				<RefreshButton />
				<SearchInput
					value={dataQuery.search}
					onChange={(search) => setDataQuery({ search })}
				/>
				<GamesSortMenu />
			</Stack>
			<FilterList />
		</>
	);
}
