import { Game } from "@api/bindings";
import styles from "./game-tags.module.css";

type Props = {
	readonly game: Game;
};

export function GameTags({ game }: Props) {
	return (
		<div className={styles.wrapper}>
			{game.tags.sort().map((tag) => (
				<span
					className={styles.tag}
					key={tag}
				>
					{tag}
				</span>
			))}
		</div>
	);
}
