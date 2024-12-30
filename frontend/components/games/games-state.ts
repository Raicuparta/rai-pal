import { atom, useAtomValue } from "jotai";
import { GamesColumnId, gamesColumns } from "./games-columns";
import { ProviderId } from "@api/bindings";

export const selectedGameAtom = atom<{
	gameId: string;
	providerId: ProviderId;
} | null>(null);

export const visibleGamesColumnsAtom = atom<GamesColumnId[]>([
	"thumbnail",
	"name",
	"engine",
	"architecture",
]);

export const useVisibleGamesColumns = () => {
	const visibleColumnIds = useAtomValue(visibleGamesColumnsAtom);
	// TODO filter columns.
	return gamesColumns;
	return gamesColumns.filter((column) => visibleColumnIds.includes(column.id));
};
