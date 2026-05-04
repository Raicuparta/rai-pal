import { ModLoaderId, ModLoaderStatus } from "@api/bindings";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "./use-data";

export type UnifiedModLoader = {
	id: ModLoaderId;
	status?: ModLoaderStatus;
};

export function useUnifiedModLoaders(
	statuses: Partial<Record<ModLoaderId, ModLoaderStatus>>,
) {
	const modLoaders = useAtomValue(modLoadersAtom);

	return Object.values(modLoaders)
		.filter((modLoader) => modLoader.kind === "Installable")
		.sort((a, b) => a.id.localeCompare(b.id))
		.map((modLoader) => ({
			id: modLoader.id,
			status: statuses[modLoader.id],
		}));
}
