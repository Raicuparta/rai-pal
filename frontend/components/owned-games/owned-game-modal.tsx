import { Button, Modal } from "@mantine/core";
import { OwnedUnityGame } from "@api/bindings";
import { open } from "@tauri-apps/api/shell";
import { CommandButton } from "@components/command-button";
import { MdInstallDesktop, MdWeb } from "react-icons/md";

type Props = {
  readonly selectedGame: OwnedUnityGame;
  readonly onClose: () => void;
};

export function OwnedGameModal(props: Props) {
  return (
    <Modal
      title={props.selectedGame.name}
      centered
      opened
      onClose={props.onClose}
    >
      <Button.Group orientation="vertical">
        <CommandButton
          icon={<MdWeb />}
          onClick={() =>
            open(`https://steampowered.com/app/${props.selectedGame.id}`)
          }
        >
          Open Steam Page
        </CommandButton>
        <CommandButton
          icon={<MdInstallDesktop />}
          onClick={() => open(`steam://install/${props.selectedGame.id}`)}
        >
          Install on Steam
        </CommandButton>
      </Button.Group>
    </Modal>
  );
}
