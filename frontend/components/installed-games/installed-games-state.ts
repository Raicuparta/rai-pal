import { atom, useAtomValue } from "jotai";
import { InstalledGameId } from "./installed-games-page";
import {
	InstalledGameColumnsId,
	installedGamesColumns,
} from "./installed-games-columns";

export const selectedInstalledGameAtom = atom<InstalledGameId | null>(null);

export const visibleInstalledGameColumnsAtom = atom<InstalledGameColumnsId[]>([
	"thumbnail",
	"name",
	"engine",
	"provider",
]);

export const useVisibleInstalledGameColumns = () => {
	const visibleColumnIds = useAtomValue(visibleInstalledGameColumnsAtom);
	return installedGamesColumns.filter((column) =>
		visibleColumnIds.includes(column.id),
	);
};
