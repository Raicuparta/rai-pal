import { Alert, Button, Code, Modal, Stack } from "@mantine/core";
import { useModLoaders } from "@hooks/use-backend-data";
import {
  Game,
  installMod,
  openGameFolder,
  openGameModsFolder,
  startGame,
  uninstallMod,
} from "@api/bindings";
import { Fragment, useState } from "react";
import { GameName } from "./game-name";
import { CommandButton } from "@components/command-button";
import {
  MdDelete,
  MdFolderSpecial,
  MdInstallDesktop,
  MdPlayArrow,
  MdRuleFolder,
} from "react-icons/md";

type Props = {
  readonly game: Game;
  readonly onClose: () => void;
  readonly refreshGame: (gameId: string) => void;
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
          <CommandButton
            icon={<MdPlayArrow />}
            onClick={() => startGame(props.game.id)}
          >
            Start Game
          </CommandButton>
          <CommandButton
            icon={<MdFolderSpecial />}
            onClick={() => openGameFolder(props.game.id)}
          >
            Open Game Folder
          </CommandButton>
          <CommandButton
            icon={<MdRuleFolder />}
            onClick={() => openGameModsFolder(props.game.id).catch(handleError)}
          >
            Open Mods Folder
          </CommandButton>
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
                    .map((mod) =>
                      props.game.installedMods.includes(mod.id) ? (
                        <CommandButton
                          icon={<MdDelete />}
                          key={mod.name}
                          onClick={async () => {
                            await uninstallMod(props.game.id, mod.id);
                            props.refreshGame(props.game.id);
                          }}
                        >
                          Uninstall {mod.name}
                        </CommandButton>
                      ) : (
                        <CommandButton
                          icon={<MdInstallDesktop />}
                          key={mod.name}
                          onClick={async () => {
                            await installMod(
                              modLoader.id,
                              mod.id,
                              props.game.id
                            );
                            props.refreshGame(props.game.id);
                          }}
                        >
                          Install {mod.name}
                        </CommandButton>
                      )
                    )}
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
