import { Button, Modal } from "@mantine/core";
import { OwnedUnityGame } from "@api/bindings";
import { open } from "@tauri-apps/api/shell";

type Props = {
  selectedGame: OwnedUnityGame;
  onClose: () => void;
};

export function OwnedGameModal(props: Props) {
  return (
    <Modal
      title={props.selectedGame.name}
      centered
      opened={true}
      onClose={props.onClose}
    >
      <Button.Group orientation="vertical">
        <Button
          variant="default"
          onClick={() =>
            open(`https://steampowered.com/app/${props.selectedGame.id}`)
          }
        >
          Open Steam Page
        </Button>
        <Button
          variant="default"
          onClick={() => open(`steam://install/${props.selectedGame.id}`)}
        >
          Install on Steam
        </Button>
      </Button.Group>
    </Modal>
  );
}
