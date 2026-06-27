import { convertFileSrc } from '@tauri-apps/api/core';
import { appDataDir, join } from '@tauri-apps/api/path';

/** Resolve an app-data-relative file through Tauri's scoped asset protocol. */
export async function appAssetUrl(relativePath: string): Promise<string> {
	return convertFileSrc(await join(await appDataDir(), relativePath));
}
