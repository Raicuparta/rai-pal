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
]);

export const useVisibleInstalledGameColumns = () => {
	const visibleColumnIds = useAtomValue(visibleInstalledGameColumnsAtom);
	return installedGamesColumns.filter((column) =>
		visibleColumnIds.includes(column.id),
	);
};
