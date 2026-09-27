import { ActionIcon, Divider, Group, Select, Stack } from "@mantine/core";
import { GamesSortBy } from "@api/bindings";
import { IconSortAscending, IconSortDescending } from "@tabler/icons-react";
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
	const { t: tSort } = useLocalization("gamesSort");

	const { sortBy, sortDescending } = dataQuery;

	return (
		<Stack px="xs">
			<Divider label={tSort("sortBy")} />
			<Group>
				<Select
					flex={1}
					data={sortOptions.map((option) => ({
						value: option,
						label: t(sortLocalizationKeys[option]) ?? option,
					}))}
					value={sortBy}
					allowDeselect={false}
					onChange={(value) => {
						if (!value) {
							return;
						}
						setDataQuery({
							sortBy: value as GamesSortBy,
							sortDescending: value === sortBy ? !sortDescending : false,
						});
					}}
				/>
				<ActionIcon
					size="input-xs"
					variant="default"
					aria-label={tSort("sortDirection")}
					onClick={() => setDataQuery({ sortDescending: !sortDescending })}
				>
					{sortDescending ? <IconSortDescending /> : <IconSortAscending />}
				</ActionIcon>
			</Group>
		</Stack>
	);
}
