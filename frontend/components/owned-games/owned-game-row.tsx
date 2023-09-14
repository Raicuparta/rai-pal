import { MdCheckCircle } from "react-icons/md";
import { OwnedUnityGame } from "@api/bindings";

export function OwnedGameRow(_: number, ownedUnityGame: OwnedUnityGame) {
  return (
    <>
      <td>{ownedUnityGame.name}</td>
      <td>{ownedUnityGame.installed ? <MdCheckCircle /> : ""}</td>
    </>
  );
}
