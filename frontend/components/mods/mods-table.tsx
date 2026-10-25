import { Table } from "@mantine/core";
import { GameMod, ModFamily } from "@api/bindings";
import { compareReleaseDates } from "../../date-formatter";
import { useAtomValue } from "jotai";
import { modFamiliesAtom } from "@hooks/use-data";
import { useMemo } from "react";
import { ModRow } from "./mod-row";
import { ModFamilyRow } from "./mod-family-row";

type Props = {
	readonly mods: Record<string, GameMod>;
};

type ModGroup =
	| { readonly type: "mod"; readonly mod: GameMod }
	| {
			readonly type: "family";
			readonly familyId: string;
			readonly family: ModFamily;
			readonly mods: GameMod[];
	  };

function getGroupReleaseDate(group: ModGroup) {
	if (group.type === "mod") return group.mod.download?.releaseDate;
	return group.mods[0]?.download?.releaseDate;
}

export function ModsTable(props: Props) {
	const modFamilies = useAtomValue(modFamiliesAtom);

	const groups = useMemo(() => {
		const familyMods = new Map<string, GameMod[]>();
		const result: ModGroup[] = [];

		for (const mod of Object.values(props.mods)) {
			if (mod.family) {
				const existing = familyMods.get(mod.family);
				if (existing) {
					existing.push(mod);
				} else {
					familyMods.set(mod.family, [mod]);
				}
			} else {
				result.push({ type: "mod", mod });
			}
		}

		for (const [familyId, mods] of familyMods) {
			mods.sort((a, b) =>
				compareReleaseDates(a.download?.releaseDate, b.download?.releaseDate),
			);

			const family = modFamilies[familyId];
			if (mods.length > 1 && family) {
				result.push({ type: "family", familyId, family, mods });
			} else {
				for (const mod of mods) {
					result.push({ type: "mod", mod });
				}
			}
		}

		result.sort((a, b) => {
			const aIsFamily = a.type === "family" ? 0 : 1;
			const bIsFamily = b.type === "family" ? 0 : 1;

			if (aIsFamily !== bIsFamily) return aIsFamily - bIsFamily;

			return compareReleaseDates(
				getGroupReleaseDate(a),
				getGroupReleaseDate(b),
			);
		});

		return result;
	}, [props.mods, modFamilies]);

	return (
		<Table style={{ tableLayout: "auto" }}>
			<Table.Tbody>
				{groups.map((group) =>
					group.type === "family" ? (
						<ModFamilyRow
							key={group.familyId}
							family={group.family}
							mods={group.mods}
						/>
					) : (
						<ModRow
							key={group.mod.id}
							mod={group.mod}
						/>
					),
				)}
			</Table.Tbody>
		</Table>
	);
}
