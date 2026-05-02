import { ModLoaderStatus } from "@api/bindings";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "./use-data";

export type UnifiedModLoader = {
	id: string;
	status?: ModLoaderStatus;
};

export function useUnifiedModLoaders(statuses: Record<string, ModLoaderStatus>) {
	const modLoaders = useAtomValue(modLoadersAtom);
	const modLoaderIds = Object.keys(modLoaders)
		.filter((id) => modLoaders[id]?.kind === "Installable")
		.sort();

	return modLoaderIds.map((id) => ({
		id,
		status: statuses[id],
	}));
}
