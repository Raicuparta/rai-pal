import { Alert, Button, Code, Modal, Stack } from "@mantine/core";
import { useModLoaders } from "@hooks/use-backend-data";
import {
  Game,
  installMod,
  openGameFolder,
  openGameModsFolder,
  startGame,
} from "@api/bindings";
import { Fragment, useState } from "react";
import { GameName } from "./game-executable-name";

type Props = {
  readonly game: Game;
  readonly onClose: () => void;
};

export function InstalledGameModal(props: Props) {
  const [modLoaders] = useModLoaders();
  const [error, setError] = useState("");

  const handleError = (error: unknown) => {
    setError(`${error}`);
  };

  return (
    <Modal
      opened
      onClose={props.onClose}
      title={<GameName game={props.game} />}
      centered
      size="lg"
    >
      <Stack>
        {error ? (
          <Alert color="red" sx={{ overflow: "auto", flex: 1 }}>
            <pre>{error}</pre>
          </Alert>
        ) : null}
        <Button.Group orientation="vertical">
          <Button
            variant="default"
            onClick={() => startGame(props.game.id).catch(handleError)}
          >
            Start Game
          </Button>
          <Button
            variant="default"
            onClick={() => openGameFolder(props.game.id).catch(handleError)}
          >
            Open Game Folder
          </Button>
          <Button
            variant="default"
            onClick={() => openGameModsFolder(props.game.id).catch(handleError)}
          >
            Open Mods Folder
          </Button>
        </Button.Group>
        {Object.values(modLoaders).map(
          (modLoader) =>
            // TODO need to filter these mods before checking the length.
            // Because we only show mods compatible with this game.
            modLoader.mods.length > 0 && (
              <Fragment key={modLoader.id}>
                <label>{modLoader.id} mods</label>
                <Button.Group orientation="vertical">
                  {modLoader.mods
                    .filter(
                      (mod) =>
                        mod.scriptingBackend === props.game.scriptingBackend
                    )
                    .map((mod) => (
                      <Button
                        variant="default"
                        key={mod.name}
                        onClick={() =>
                          installMod(modLoader.id, mod.id, props.game.id).catch(
                            handleError
                          )
                        }
                      >
                        Install {mod.name}
                      </Button>
                    ))}
                </Button.Group>
              </Fragment>
            )
        )}
        <label>Debug Data</label>
        <Code sx={{ overflow: "auto" }}>
          <pre>{JSON.stringify(props.game, null, 2)}</pre>
        </Code>
      </Stack>
    </Modal>
  );
}
