import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { Sidebar } from "@components/sidebar";
import { useDataQuery } from "@hooks/use-data-query";
import { GamesSortMenu } from "./games-sort-menu";
import { FilterList } from "./filters/filter-list";

export function GamesSidebar() {
	const [dataQuery, setDataQuery] = useDataQuery();

	return (
		<Sidebar>
			<RefreshButton />
			<SearchInput
				value={dataQuery.search}
				onChange={(search) => setDataQuery({ search })}
			/>
			<GamesSortMenu />
			<FilterList />
		</Sidebar>
	);
}
