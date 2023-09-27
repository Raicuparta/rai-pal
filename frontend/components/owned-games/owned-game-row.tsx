import { MdCheckCircle } from "react-icons/md";
import { OwnedUnityGame } from "@api/bindings";
import { Badge, Table } from "@mantine/core";
import { engineColor } from "../../util/color";

export function OwnedGameRow(_: number, ownedUnityGame: OwnedUnityGame) {
  return (
    <>
      <Table.Td>{ownedUnityGame.name}</Table.Td>
      <Table.Td align="center">
        <Badge color={engineColor[ownedUnityGame.engine]}>
          {ownedUnityGame.engine}
        </Badge>
      </Table.Td>
      <Table.Td align="center">
        {ownedUnityGame.osList.includes("Linux") ? <MdCheckCircle /> : ""}
      </Table.Td>
      <Table.Td align="center">
        {ownedUnityGame.installed ? <MdCheckCircle /> : ""}
      </Table.Td>
    </>
  );
}
