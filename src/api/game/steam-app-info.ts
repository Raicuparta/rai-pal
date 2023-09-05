export type AppLaunch = {
  description?: string;
  executable?: string;
  arguments?: string;
  type?: string;
  workingdir?: string;
  config?: {
    oslist?: string;
    betakey?: string;
    // Yeah this thing will some times randomly be a number and some times a string.
    osarch?: string | number;
  };
};

export type LaunchMap = Record<string, AppLaunch>;

export type AppInfo = {
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
      launch?: LaunchMap;
    };
  };
};
