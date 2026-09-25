import { Button, Menu } from "@mantine/core";
import { GamesSortBy } from "@api/bindings";
import {
	IconArrowsSort,
	IconChevronDown,
	IconSortAscending,
	IconSortDescending,
} from "@tabler/icons-react";
import { useDataQuery } from "@hooks/use-data-query";
import { useLocalization } from "@hooks/use-localization";
import { LocalizationKey } from "@localizations/localizations";

const sortOptions: GamesSortBy[] = ["Title", "Engine", "ReleaseDate"];

const sortLocalizationKeys: Record<
	GamesSortBy,
	LocalizationKey<"gamesTableColumn">
> = {
	Title: "name",
	Engine: "engine",
	ReleaseDate: "date",
};

export function GamesSortMenu() {
	const [dataQuery, setDataQuery] = useDataQuery();
	const { t } = useLocalization("gamesTableColumn");

	const { sortBy, sortDescending } = dataQuery;

	const changeSort = (newSortBy: GamesSortBy) => {
		setDataQuery({
			sortBy: newSortBy,
			sortDescending: newSortBy === sortBy ? !sortDescending : false,
		});
	};

	return (
		<Menu
			keepMounted
			withOverlay={false}
		>
			<Menu.Target>
				<Button
					leftSection={
						sortDescending ? <IconSortDescending /> : <IconSortAscending />
					}
					rightSection={<IconChevronDown />}
				>
					{t(sortLocalizationKeys[sortBy])}
				</Button>
			</Menu.Target>
			<Menu.Dropdown>
				{sortOptions.map((option) => (
					<Menu.Item
						key={option}
						onClick={() => changeSort(option)}
						leftSection={
							option === sortBy ? (
								sortDescending ? (
									<IconSortDescending />
								) : (
									<IconSortAscending />
								)
							) : (
								<IconArrowsSort opacity={0.4} />
							)
						}
					>
						{t(sortLocalizationKeys[option])}
					</Menu.Item>
				))}
			</Menu.Dropdown>
		</Menu>
	);
}
