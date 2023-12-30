import { ProviderId } from "@api/bindings";

export function getFallbackThumbnail(provider: ProviderId) {
	return `images/thumbnails/${provider}.png`;
}

export function getThumbnailWithFallback(
	url: string | null | undefined,
	provider: ProviderId,
) {
	return url || getFallbackThumbnail(provider);
}
