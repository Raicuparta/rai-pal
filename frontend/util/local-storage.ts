import { showAppNotification } from "@components/app-notifications";

const STORAGE_RESET_EVENT_NAME = "storage-reset";
const STORAGE_CHANGE_EVENT_NAME_PREFIX = "storage-changed";

// Increment this to invalidate previous incompatible versions.
const storageVersion = 1;

function getVersionedKey(key: string) {
	return `${key}-v${storageVersion}`;
}

function getEventId(key: string) {
	return `${STORAGE_CHANGE_EVENT_NAME_PREFIX}-${key}`;
}

export function setLocalStorage<TValue>(key: string, value: TValue) {
	if (!value) {
		localStorage.removeItem(getVersionedKey(key));
	} else {
		localStorage.setItem(getVersionedKey(key), JSON.stringify(value));
	}

	window.dispatchEvent(new Event(getEventId(key)));
}

export function listenToStorageChange<TValue>(
	key: string,
	defaultValue: TValue,
	callback: (value: TValue) => void,
) {
	function handleStorageChange() {
		callback(getLocalStorage(key, defaultValue));
	}

	const eventId = getEventId(key);

	window.addEventListener(eventId, handleStorageChange);
	window.addEventListener(STORAGE_RESET_EVENT_NAME, handleStorageChange);

	return () => {
		window.removeEventListener(eventId, handleStorageChange);
		window.removeEventListener(STORAGE_RESET_EVENT_NAME, handleStorageChange);
	};
}

export function getLocalStorage<TValue>(
	key: string,
	defaultValue: TValue,
): TValue {
	try {
		const storageValue = localStorage.getItem(getVersionedKey(key));
		return storageValue ? JSON.parse(storageValue) : defaultValue;
	} catch (error) {
		console.error(
			`Failed to parse local storage JSON for key "${key}":`,
			error,
		);
		return defaultValue;
	}
}

export function resetLocalStorage() {
	localStorage.clear();
	window.dispatchEvent(new Event(STORAGE_RESET_EVENT_NAME));
	showAppNotification("Cleared local storage", "success");
}
