import { ProviderId } from "@api/bindings";
import {
	Icon,
	IconDeviceGamepad,
	IconBrandSteam,
	IconSquareLetterE,
	IconCircleLetterG,
	IconBrandXbox,
} from "@tabler/icons-react";

type Props = {
	readonly providerId: ProviderId;
};

const providerIcons: Record<ProviderId, Icon> = {
	Manual: IconDeviceGamepad,
	Steam: IconBrandSteam,
	Epic: IconSquareLetterE,
	Gog: IconCircleLetterG,
	Xbox: IconBrandXbox,
};

export function ProviderIcon(props: Props) {
	const IconComponent = providerIcons[props.providerId] ?? IconDeviceGamepad;
	return <IconComponent />;
}
