import { ActionIcon, Group, Stack, Text, ThemeIcon } from "@mantine/core";
import { FilterGroup, FilterItem, GamesFilter } from "@api/bindings";
import { IconLock, IconLockOpen, IconRestore } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { CheckboxButton } from "@components/checkbox-button";
import { filterDetails } from "./filter-menu";

export type FilterKey = keyof GamesFilter;
export type FilterChangeCallback = (
	id: FilterKey,
	values: FilterGroup<string>,
) => void;

export function keepOnlyLocked(
	group: FilterGroup<string>,
): FilterGroup<string> {
	const known = Object.fromEntries(
		Object.entries(group.known).filter(([, item]) => item.locked),
	);
	const unknown = group.unknown && group.unknown.locked ? group.unknown : null;
	return { known, unknown };
}

type Props<TFilterKey extends FilterKey> = {
	readonly id: TFilterKey;
	readonly possibleValues: Array<string>;
	readonly filterGroup: FilterGroup<string>;
	readonly onChange: FilterChangeCallback;
};

function getDefaultItem(): FilterItem {
	return { enabled: true, locked: false };
}

function getItem(known: Record<string, FilterItem>, key: string): FilterItem {
	return known[key] ?? getDefaultItem();
}

export function FilterSelect<TFilterKey extends FilterKey>({
	id,
	possibleValues,
	filterGroup,
	onChange,
}: Props<TFilterKey>) {
	const { t: tProperty } = useLocalization("filterProperty");
	const { t: tValue } = useLocalization("filterValue");
	const { t: tValueNote } = useLocalization("filterValueNote");
	const emptyLocalizationKey = filterDetails[id].emptyLocalizationKey;
	const possibleValuesWithNull = [
		...(emptyLocalizationKey ? [""] : []),
		...possibleValues,
	];

	function modifyKnown(
		key: string,
		updater: (prev: FilterItem) => FilterItem,
	): FilterGroup<string> {
		if (key === "" && emptyLocalizationKey) {
			const prev = filterGroup.unknown ?? getDefaultItem();
			const next = updater(prev);
			if (next.enabled && !next.locked) {
				return { known: filterGroup.known, unknown: null };
			}
			return { known: filterGroup.known, unknown: next };
		}
		const prev = getItem(filterGroup.known, key);
		const next = updater(prev);
		const nextKnown = { ...filterGroup.known };
		if (next.enabled && !next.locked) {
			delete nextKnown[key];
		} else {
			nextKnown[key] = next;
		}
		return { known: nextKnown, unknown: filterGroup.unknown };
	}

	function handleFilterClick(key: string) {
		onChange(
			id,
			modifyKnown(key, (prev) => ({
				...prev,
				enabled: !prev.enabled,
			})),
		);
	}

	function handleLockClick(key: string) {
		onChange(
			id,
			modifyKnown(key, (prev) => ({
				...prev,
				locked: !prev.locked,
			})),
		);
	}

	function handleExclusiveClick(key: string) {
		// Enable only this value, preserve locked items
		const outKnown: Record<string, FilterItem> = {};
		let outUnknown: FilterItem | null = filterGroup.unknown;

		// Keep locked items as-is
		for (const k of Object.keys(filterGroup.known)) {
			const item = filterGroup.known[k]!;
			if (item.locked) {
				outKnown[k] = { ...item };
			}
		}

		// If the clicked value is locked, make sure it's enabled
		if (key === "" && emptyLocalizationKey) {
			const current = filterGroup.unknown ?? getDefaultItem();
			if (current.locked) {
				outUnknown = { enabled: true, locked: true };
			}
		} else if (getItem(filterGroup.known, key).locked) {
			outKnown[key] = { enabled: true, locked: true };
		}

		// Disable all non-locked values except the clicked one
		for (const k of possibleValuesWithNull) {
			if (k === key) continue;
			if (k === "" && emptyLocalizationKey) {
				const current = filterGroup.unknown ?? getDefaultItem();
				if (!current.locked) {
					outUnknown = { enabled: false, locked: false };
				}
			} else if (!getItem(filterGroup.known, k).locked) {
				outKnown[k] = { enabled: false, locked: false };
			}
		}

		onChange(id, { known: outKnown, unknown: outUnknown });
	}

	function handleResetClick() {
		onChange(id, keepOnlyLocked(filterGroup));
	}

	const unknownItem = filterGroup.unknown ?? getDefaultItem();
	const hasAnyDisabled =
		Object.values(filterGroup.known).some(
			(item) => !item.enabled && !item.locked,
		) ||
		(!unknownItem.enabled && !unknownItem.locked);

	return (
		<Stack>
			<Stack gap={5}>
				<Group wrap="nowrap">
					{!hasAnyDisabled ? (
						<ThemeIcon
							size="sm"
							variant="transparent"
							color="gray"
							opacity={0.3}
						>
							<IconRestore fontSize={13} />
						</ThemeIcon>
					) : (
						<ActionIcon
							size="sm"
							variant="subtle"
							onClick={handleResetClick}
						>
							<IconRestore fontSize={13} />
						</ActionIcon>
					)}
					<Text fz="md">{tProperty(filterDetails[id].localizationKey)}</Text>
				</Group>
				<Stack
					gap={2}
					miw={100}
				>
					{possibleValues.map((possibleValue) => {
						const valueDetails = filterDetails[id].valueDetails[possibleValue];
						const item = getItem(filterGroup.known, possibleValue);

						return (
							<Group
								key={possibleValue}
								gap={2}
								wrap="nowrap"
							>
								<ActionIcon
									size="sm"
									variant="subtle"
									color={item.locked ? "yellow" : "gray"}
									disabled={item.enabled}
									onClick={() => handleLockClick(possibleValue)}
								>
									{item.locked ? (
										<IconLock size={14} />
									) : (
										<IconLockOpen size={14} />
									)}
								</ActionIcon>
								<CheckboxButton
									tooltip={tValueNote(valueDetails?.noteLocalizationKey)}
									checked={item.enabled}
									disabled={item.locked && !item.enabled}
									onClickCheckbox={() => handleFilterClick(possibleValue)}
									onClickButton={() => handleExclusiveClick(possibleValue)}
								>
									{valueDetails?.staticDisplayText ??
										tValue(valueDetails?.localizationKey) ??
										possibleValue}
								</CheckboxButton>
							</Group>
						);
					})}
					{emptyLocalizationKey && (
						<Group
							gap={2}
							wrap="nowrap"
						>
							<ActionIcon
								size="sm"
								variant="subtle"
								color={unknownItem.locked ? "yellow" : "gray"}
								disabled={unknownItem.enabled}
								onClick={() => handleLockClick("")}
							>
								{unknownItem.locked ? (
									<IconLock size={14} />
								) : (
									<IconLockOpen size={14} />
								)}
							</ActionIcon>
							<CheckboxButton
								checked={unknownItem.enabled}
								disabled={unknownItem.locked && !unknownItem.enabled}
								onClickCheckbox={() => handleFilterClick("")}
								onClickButton={() => handleExclusiveClick("")}
							>
								{tValue(emptyLocalizationKey)}
							</CheckboxButton>
						</Group>
					)}
				</Stack>
			</Stack>
		</Stack>
	);
}
