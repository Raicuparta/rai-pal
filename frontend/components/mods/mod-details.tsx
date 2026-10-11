import { Group, Stack, Text } from "@mantine/core";
import {
	EngineVersionNumbers,
	EngineVersionRange,
	GameMod,
} from "@api/bindings";
import {
	IconCalendarFilled,
	IconGizmo,
	IconUserFilled,
} from "@tabler/icons-react";
import { ReactNode } from "react";
import { DeprecatedBadge } from "./deprecated-badge";
import { dateFormatter } from "../../date-formatter";

type Props = {
	readonly mod: GameMod;
	readonly versionText?: string;
	readonly leading?: ReactNode;
	readonly titleAdornment?: ReactNode;
};

function renderVersionRange(range: EngineVersionRange | null) {
	if (!range) return null;

	return [range.minimum, range.maximum]
		.map(renderVersionNumbers)
		.filter(Boolean)
		.join(" - ");
}

function renderVersionNumbers(version: EngineVersionNumbers | null) {
	if (!version) return null;

	return [version.major, version.minor, version.patch]
		.filter(Boolean)
		.join(".");
}

export function ModDetails({
	mod,
	versionText,
	leading,
	titleAdornment,
}: Props) {
	const engineText = [
		mod.engine,
		renderVersionRange(mod.engineVersionRange),
		mod.unityBackend,
		mod.gameOs,
		mod.architecture,
	]
		.filter(Boolean)
		.join(" ");

	return (
		<Group
			wrap="nowrap"
			align="center"
		>
			{leading}
			<Stack gap={0}>
				<Group>
					<Text
						fw="bold"
						size="lg"
					>
						{mod.title}
					</Text>
					{titleAdornment}
				</Group>
				<Group
					gap={0}
					style={{ columnGap: 10 }}
					fz="xs"
					opacity={0.4}
				>
					{versionText && <span>{versionText}</span>}
					{mod.download?.releaseDate && (
						<Group gap={2}>
							<IconCalendarFilled fontSize={10} />
							<span>
								{dateFormatter.format(new Date(mod.download.releaseDate))}
							</span>
						</Group>
					)}
					{mod.author && (
						<Group gap={2}>
							<IconUserFilled fontSize={10} />
							<span>{mod.author}</span>
						</Group>
					)}
					{engineText && (
						<Group gap={2}>
							<IconGizmo fontSize={10} />
							<span>{engineText}</span>
						</Group>
					)}
				</Group>

				<Text
					fz="sm"
					opacity={0.75}
				>
					{mod.description}
				</Text>
			</Stack>
			{mod.deprecated && <DeprecatedBadge size="sm" />}
		</Group>
	);
}
