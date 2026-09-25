import { Stack } from "@mantine/core";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { useDataQuery } from "@hooks/use-data-query";
import { GamesSortMenu } from "./games-sort-menu";
import { FilterList } from "./filters/filter-list";

export function GamesSidebar() {
	const [dataQuery, setDataQuery] = useDataQuery();

	return (
		<Stack
			flex="0 0 auto"
			gap="xs"
			p="xs"
		>
			<RefreshButton />
			<SearchInput
				value={dataQuery.search}
				onChange={(search) => setDataQuery({ search })}
			/>
			<GamesSortMenu />
			<FilterList />
		</Stack>
	);
}
