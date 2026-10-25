import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { useDataQuery } from "@hooks/use-data-query";
import { GamesSortMenu } from "./games-sort-menu";
import { FilterList } from "./filters/filter-list";
import { Box } from "@mantine/core";

export function GamesSidebar() {
	const [dataQuery, setDataQuery] = useDataQuery();

	return (
		<>
			<Box pr="xs">
				<RefreshButton />
			</Box>
			<SearchInput
				value={dataQuery.search}
				onChange={(search) => setDataQuery({ search })}
			/>
			<GamesSortMenu />
			<FilterList />
		</>
	);
}
