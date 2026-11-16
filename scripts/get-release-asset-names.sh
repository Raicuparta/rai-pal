#!/usr/bin/env bash

release_asset_targets() {
	local platform="$1"
	local output_dir="$2"
	local prefix=""

	if [[ -n "$output_dir" ]]; then
		prefix="$output_dir/"
	fi

	MAIN_TARGET=""
	PORTABLE_TARGET=""
	UPDATER_TARGET=""
	PORTABLE_UPDATER_TARGET=""
	PLATFORM_KEY=""
	PORTABLE_PLATFORM_KEY=""

	case "$platform" in
	windows)
		MAIN_TARGET="${prefix}RaiPal.exe"
		PORTABLE_TARGET="${prefix}RaiPal.portable.zip"
		UPDATER_TARGET="${prefix}updater-windows.zip"
		PORTABLE_UPDATER_TARGET="${prefix}updater-windows-portable.zip"
		PLATFORM_KEY="windows-x86_64"
		PORTABLE_PLATFORM_KEY="windows-x86_64-portable"
		;;
	linux)
		MAIN_TARGET="${prefix}RaiPal.AppImage"
		UPDATER_TARGET="${prefix}updater-linux.tar.gz"
		PLATFORM_KEY="linux-x86_64"
		;;
	macos)
		local arch
		arch="$(uname -m)"
		if [[ "$arch" == "arm64" ]]; then
			arch="aarch64"
		fi
		MAIN_TARGET="${prefix}RaiPal.macos.zip"
		UPDATER_TARGET="${prefix}updater-macos.tar.gz"
		PLATFORM_KEY="darwin-$arch"
		;;
	*)
		return 1
		;;
	esac
}

release_asset_names() {
	local platform="$1"
	release_asset_targets "$platform" "" || return 1
	local var
	for var in MAIN_TARGET PORTABLE_TARGET UPDATER_TARGET PORTABLE_UPDATER_TARGET; do
		if [[ -n "${!var}" ]]; then
			echo "${!var}"
		fi
	done
}
