import { Button, Modal, Stack } from "@mantine/core";
import { GameExecutableData } from "./game-executable-row";
import { useModLoaders } from "@hooks/use-backend-data";
import { installMod } from "@api/bindings";
import { Fragment } from "react";
import { getGameExecutableNameSuffix } from "./game-name-suffix";
import { GameExecutableName } from "./game-executable-name";

type Props = {
  data: GameExecutableData;
};

export const ModInstallModal = (props: Props) => {
  const [modLoaders] = useModLoaders();

  return (
    <Modal
      opened={true}
      onClose={() => props.data.installMod(undefined)}
      title={<GameExecutableName data={props.data} />}
      centered
      size="lg"
    >
      {modLoaders.map((modLoader) => (
        <Fragment key={modLoader.id}>
          <p>{modLoader.id} mods:</p>
          <Stack>
            {modLoader.mods
              .filter(
                (mod) =>
                  mod.scriptingBackend ===
                  props.data.executable.scriptingBackend
              )
              .map((mod) => (
                <Button
                  variant="default"
                  key={mod.name}
                  onClick={() =>
                    installMod(
                      modLoader.id,
                      mod.id,
                      props.data.game.id,
                      props.data.executable.id
                    )
                  }
                >
                  Install {mod.name}
                </Button>
              ))}
          </Stack>
        </Fragment>
      ))}
    </Modal>
  );
};
