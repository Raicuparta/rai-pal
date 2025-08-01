import { Table } from "@mantine/core";
import React from "react";
import { useSetAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { useGame } from "@hooks/use-game";
import { ItemProps } from "react-virtuoso";
import { DbGame, ProviderId } from "@api/bindings";
import { gamesColumns } from "./games-columns";
import { useAppSettings } from "@hooks/use-app-settings";

// Needs to be consistent with height set in table.module.css ugh. TODO: fix that.
export const gameRowHeight = 60;

export const GameRow = React.forwardRef(function GameRow(
	props: ItemProps<[ProviderId, string]>,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const [providerId, gameId] = props.item;
	const game = useGame(providerId, gameId);
	const setSelectedGame = useSetAtom(selectedGameAtom);

	const defaultGame: DbGame = {
		providerId: providerId,
		gameId: gameId,
		displayTitle: "...",
		engineBrand: null,
		engineVersionMajor: null,
		engineVersionMinor: null,
		engineVersionPatch: null,
		engineVersionDisplay: null,
		exePath: null,
		externalId: "",
		releaseDate: null,
		thumbnailUrl: null,
		architecture: null,
		unityBackend: null,
		titleDiscriminator: null,
		providerCommands: {},
		tags: [],
	};

	return (
		<GameRowInner
			game={game || defaultGame}
			ref={ref}
			onClick={() => setSelectedGame({ providerId, gameId })}
		/>
	);
});

type Props = { readonly game: DbGame; readonly onClick?: () => void };

export const GameRowInner = React.forwardRef(function GameRowInner(
	props: Props,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const [settings] = useAppSettings();

	return (
		<Table.Tr
			ref={ref}
			onClick={props.onClick}
		>
			{gamesColumns.map(
				(column) =>
					(!settings.hideGameThumbnails || column.id !== "thumbnail") && (
						<React.Fragment key={column.id}>
							<column.component item={props.game} />
						</React.Fragment>
					),
			)}
		</Table.Tr>
	);
});
