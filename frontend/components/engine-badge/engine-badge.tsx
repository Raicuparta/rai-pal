import { EngineBrand, Game } from "@api/bindings";
import { DefaultMantineColor, Box } from "@mantine/core";
import styles from "./engine-badge.module.css";

const engineColors: Record<EngineBrand, DefaultMantineColor> = {
	Unity: "blue",
	Unreal: "red",
	Godot: "violet",
	GameMaker: "teal",
} as const;

type Props = {
	readonly game: Game;
};

export function EngineBadge({ game }: Props) {
	const engine =
		game.installedGame?.executable.engine ?? game.remoteGame?.engine;

	const engineColor = engine ? engineColors[engine.brand] : "gray";

	const scriptingBackend = game.installedGame?.executable.scriptingBackend;

	const architecture = game.installedGame?.executable.architecture;

	const detailsText =
		scriptingBackend && architecture
			? `${scriptingBackend} ${architecture}`
			: architecture;

	return (
		<Box
			bg={engine ? `var(--mantine-color-${engineColor}-light)` : undefined}
			className={styles.wrapper}
		>
			<Box
				c={`var(--mantine-color-${engineColor}-light-color)`}
				className={styles.brand}
			>
				{engine?.brand}
			</Box>
			{engine?.version?.display && (
				<Box className={styles.version}>{engine.version.display}</Box>
			)}
			{detailsText && <Box className={styles.details}>{detailsText}</Box>}
		</Box>
	);
}
