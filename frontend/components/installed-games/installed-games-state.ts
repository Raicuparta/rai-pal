import { atom, useAtomValue } from "jotai";
import {
	InstalledGameColumnsId,
	installedGamesColumns,
} from "./installed-games-columns";
import { ProviderId } from "@api/bindings";

export const selectedInstalledGameAtom = atom<{
	gameId: string;
	providerId: ProviderId;
} | null>(null);

export const visibleInstalledGameColumnsAtom = atom<InstalledGameColumnsId[]>([
	"thumbnail",
	"name",
	"engine",
	"provider",
	"architecture",
	"gameTags",
]);

export const useVisibleInstalledGameColumns = () => {
	const visibleColumnIds = useAtomValue(visibleInstalledGameColumnsAtom);
	// TODO filter columns.
	// return installedGamesColumns;
	return installedGamesColumns.filter((column) =>
		visibleColumnIds.includes(column.id),
	);
};
