/** Mirrors `repository` in `src-tauri/Cargo.toml` and the updater endpoint in `tauri.conf.json`. */
export const REPO_URL = 'https://github.com/Cheng-Ching-Wu/PinNote'

export const APP_LINKS = {
  source: REPO_URL,
  releases: `${REPO_URL}/releases`,
  issues: `${REPO_URL}/issues`,
  license: `${REPO_URL}/blob/main/LICENSE`,
} as const
