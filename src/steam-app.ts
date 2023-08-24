export type AppLaunch = {
  executable?: string;
  app_type?: string;
  os_list?: string;
};

export type SteamApp = {
  name: string;
  launch_map: Record<string, AppLaunch>;
  install_path: String;
};

export type AppMap = Record<number, SteamApp>;
