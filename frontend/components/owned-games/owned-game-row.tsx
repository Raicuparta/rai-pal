import { MdCheckCircle } from "react-icons/md";
import { OwnedUnityGame } from "@api/bindings";
import { Table } from "@mantine/core";

export function OwnedGameRow(_: number, ownedUnityGame: OwnedUnityGame) {
  return (
    <>
      <Table.Td>{ownedUnityGame.name}</Table.Td>
      <Table.Td>
        {ownedUnityGame.osList.includes("Linux") ? <MdCheckCircle /> : ""}
      </Table.Td>
      <Table.Td>{ownedUnityGame.installed ? <MdCheckCircle /> : ""}</Table.Td>
    </>
  );
}
