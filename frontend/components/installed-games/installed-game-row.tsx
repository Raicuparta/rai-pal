import { Game } from "@api/bindings";
import { TableHeader } from "@components/table/table-head";
import React from "react";

export function InstalledGameRow(headers: TableHeader<Game>[]) {
	const InstalledGameRow = (_: number, game: Game) => (
		<>
			{headers.map((header) => (
				<React.Fragment key={header.id}>
					{header.renderCell ? header.renderCell(game) : "TODO"}
				</React.Fragment>
			))}
		</>
	);

	return InstalledGameRow;
}
