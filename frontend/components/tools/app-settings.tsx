import { Locale } from "@api/bindings";
import { useAppSettings } from "@hooks/use-app-settings";
import { Select, Stack, Switch } from "@mantine/core";
import { localeToTranslation } from "../../translations/translations";

const locales: Locale[] = [
  "DeDe",
  "EnUs",
  "EsEs",
  "FrFr",
  "JaJp",
  "KoKr",
  "PtPt",
  "ZhCh",
];

const current = ""

export function AppSettings() {
  const [settings, setSettings] = useAppSettings();

  return <Stack>
    <Switch label="Show game thumbnails on list" checked={!settings.hideGameThumbnails} onChange={(event) => {
      setSettings({
        ...settings,
        hideGameThumbnails: !event.currentTarget.checked,
      })
    }} />
    <Select label="Language" value={settings.overrideLanguage ?? ""} data={[
      {
        value: "",
        label: `Auto-detect (${current})`,
      },
      ...locales.map(locale => ({
        value: locale,
        label: localeToTranslation[locale].meta.nativeName,
      }))
      ]} onChange={(value) => {
        setSettings({
          ...settings,
          overrideLanguage: (value || null) as Locale | null,
        })
      }} />
  </Stack>
}