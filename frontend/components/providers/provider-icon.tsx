import { ProviderId } from "@api/bindings";
import {
	Icon,
	IconDeviceGamepad,
	IconBrandSteam,
	IconSquareLetterE,
	IconCircleLetterG,
	IconBrandXbox,
	IconBrandItch,
	IconCircleLetterUFilled,
	IconBrandElectronicArts,
} from "@tabler/icons-react";
import { ComponentProps } from "react";

interface Props extends ComponentProps<typeof IconDeviceGamepad> {
	readonly providerId: ProviderId;
}

const providerIcons: Record<ProviderId, Icon> = {
	Manual: IconDeviceGamepad,
	Steam: IconBrandSteam,
	Epic: IconSquareLetterE,
	Gog: IconCircleLetterG,
	Xbox: IconBrandXbox,
	Itch: IconBrandItch,
	Ubisoft: IconCircleLetterUFilled,
	Ea: IconBrandElectronicArts,
};

export function ProviderIcon({ providerId, ...props }: Props) {
	const IconComponent = providerIcons[providerId] ?? IconDeviceGamepad;
	return <IconComponent {...props} />;
}
