export type SteamLaunchOption = {
  launchId: string;
  appId: number;
  description?: string;
  executable?: string;
  arguments?: string;
  appType?: string;
  osList?: string;
  betaKey?: string;
  osArch?: string;
};

export type SteamLaunchMap = Record<string, SteamLaunchOption>;

export type SteamAppInfo = {
  appinfo: {
    appid: number;
    common: {
      name: string;
      oslist: string;
      type: string;
    };
    config: {
      contenttype: number;
      installdir: string;
      launch?: SteamLaunchMap;
    };
  };
};
