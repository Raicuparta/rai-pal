export type PropsStylable<T = unknown> = T & { readonly className?: string };
export type PropsStylableWithChildren<T = unknown> = PropsStylable<T> & {
	readonly children?: React.ReactNode;
};
