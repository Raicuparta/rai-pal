import { Accordion, ActionIcon, Flex, Indicator, Stack } from "@mantine/core";
import { FilterGroup, FilterItem } from "@api/bindings";
import { IconRestore } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { CheckboxButton } from "@components/checkbox-button";
import { filterDetails, FilterKey } from "./filter-details";
import styles from "./filter-list.module.css";

export type FilterChangeCallback = (
	id: FilterKey,
	values: FilterGroup<string>,
) => void;

export function keepOnlyLocked(
	group: FilterGroup<string>,
): FilterGroup<string> {
	const known: Record<string, FilterItem> = {};
	for (const [key, item] of Object.entries(group.known)) {
		if (item?.locked) {
			known[key] = item;
		}
	}
	const unknown = group.unknown?.locked ? group.unknown : null;
	return { known, unknown };
}

export function hasLocked(group: FilterGroup<string>): boolean {
	return (
		Object.values(group.known).some((item) => item?.locked) ||
		group.unknown?.locked === true
	);
}

export function hasDisabledNonLocked(group: FilterGroup<string>): boolean {
	return (
		Object.values(group.known).some(
			(item) => item !== undefined && !item.enabled && !item.locked,
		) ||
		(group.unknown !== null && !group.unknown.enabled && !group.unknown.locked)
	);
}

type Props<TFilterKey extends FilterKey> = {
	readonly id: TFilterKey;
	readonly possibleValues: Array<string>;
	readonly filterGroup: FilterGroup<string>;
	readonly expanded: boolean;
	readonly onExpandedChange: (expanded: boolean) => void;
	readonly onChange: FilterChangeCallback;
};

function getDefaultItem(): FilterItem {
	return { enabled: true, locked: false };
}

function getItem(
	known: Record<string, FilterItem | undefined>,
	key: string,
): FilterItem {
	return known[key] ?? getDefaultItem();
}

export function FilterColumn<TFilterKey extends FilterKey>({
	id,
	possibleValues,
	filterGroup,
	expanded,
	onExpandedChange,
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

	const hasChanges = hasDisabledNonLocked(filterGroup);
	const hasLockedValues = hasLocked(filterGroup);

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

	function hasAnyEnabledValue(group: FilterGroup<string>): boolean {
		return possibleValuesWithNull.some((value) => {
			const item =
				value === "" && emptyLocalizationKey
					? group.unknown
					: group.known[value];
			const resolved = item ?? getDefaultItem();
			return resolved.enabled || resolved.locked;
		});
	}

	function handleFilterClick(key: string) {
		const next = modifyKnown(key, (prev) => ({
			...prev,
			enabled: !prev.enabled,
		}));
		onChange(id, hasAnyEnabledValue(next) ? next : keepOnlyLocked(filterGroup));
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
			outUnknown = current.locked ? { enabled: true, locked: true } : null;
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

	return (
		<Accordion
			className={styles.filterColumn}
			value={expanded ? id : null}
			onChange={(value) => onExpandedChange(value === id)}
		>
			<Accordion.Item
				value={id}
				bd={0}
			>
				<Indicator
					color="yellow.5"
					position="top-start"
					offset={8}
					size={3}
					disabled={!hasLockedValues}
				>
					<Flex
						align="stretch"
						gap={0}
						wrap="nowrap"
						pl="xs"
					>
						<ActionIcon
							size="sm"
							variant="subtle"
							disabled={!hasChanges}
							bg="transparent"
							onClick={handleResetClick}
							flex="0 0 auto"
							h="auto"
						>
							<IconRestore fontSize={16} />
						</ActionIcon>
						<Accordion.Control
							className={styles.filterTitle}
							fz="xs"
						>
							{tProperty(filterDetails[id].localizationKey)}
						</Accordion.Control>
					</Flex>
				</Indicator>
				<Accordion.Panel>
					<Stack gap={2}>
						{possibleValues.map((possibleValue) => {
							const valueDetails =
								filterDetails[id].valueDetails[possibleValue];
							const item = getItem(filterGroup.known, possibleValue);
							const tooltip = tValueNote(valueDetails?.noteLocalizationKey);

							return (
								<CheckboxButton
									key={possibleValue}
									tooltip={tooltip}
									locked={item.locked}
									checked={item.enabled}
									onClickLock={() => handleLockClick(possibleValue)}
									onClickCheckbox={() => handleFilterClick(possibleValue)}
									onClickButton={() => handleExclusiveClick(possibleValue)}
								>
									{valueDetails?.staticDisplayText ??
										tValue(valueDetails?.localizationKey) ??
										possibleValue}
								</CheckboxButton>
							);
						})}
						{emptyLocalizationKey && (
							<CheckboxButton
								locked={unknownItem.locked}
								checked={unknownItem.enabled}
								onClickLock={() => handleLockClick("")}
								onClickCheckbox={() => handleFilterClick("")}
								onClickButton={() => handleExclusiveClick("")}
							>
								{tValue(emptyLocalizationKey)}
							</CheckboxButton>
						)}
					</Stack>
				</Accordion.Panel>
			</Accordion.Item>
		</Accordion>
	);
}
