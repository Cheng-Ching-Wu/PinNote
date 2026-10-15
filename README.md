# PinNote

輕量的 Windows 桌面便籤。每張便籤都是獨立的 Tauri 視窗，內容、位置、尺寸、顏色、置頂與收合狀態都會自動存進 SQLite，重開後原樣還原。

## 下載與安裝

到 [Releases](https://github.com/Cheng-Ching-Wu/PinNote/releases) 下載最新版的 `PinNote_x.y.z_x64-setup.exe`（或 `.msi`）安裝，支援 Windows 10 / 11（需要 WebView2，Windows 11 已內建）。

主要功能：多張獨立便籤視窗、置頂與收合、Rich text（清單、表格、圖片）、標籤與搜尋、封存 / 垃圾桶、歷史版本、便籤互相連結、透過雲端硬碟資料夾跨裝置同步、匯出 / 匯入、自訂全域快捷鍵、繁體中文 / English 介面（設定 → 語言）。詳見下方「行為說明」。

從 v0.3.0 起，App 會自動檢查新版本並提示更新（設定 →「應用程式說明…」可手動檢查更新、開關自動檢查）。v0.2.0 以前的版本沒有這個功能，需要手動下載安裝一次 v0.3.0。

> 目前的安裝檔尚未數位簽章，第一次執行時 Windows SmartScreen 可能顯示「Windows 已保護您的電腦」，請點「其他資訊」→「仍要執行」。

## 開發

需求：Node 20+、Rust stable、Visual Studio Build Tools（C++ workload + Windows SDK）、WebView2。

```bash
npm install
npm run tauri:dev      # 開發模式
npm run tauri:build    # 打包安裝檔
npm test               # 前端單元測試 (vitest)
cd src-tauri && cargo test   # Rust 單元測試（migration / repository）
```

資料庫位置：`%APPDATA%\com.pinnote.app\pinnote.db`

## 架構

```text
Vue Component (views / components)
↓
Pinia Store (stores/note.ts)
↓
Repository / Service (repositories/noteRepository.ts, services/*.ts)
↓
Tauri Command (src-tauri/src/commands)
↓
Rust service → note_repository → SQLite (rusqlite)
```

- Component 不直接 `invoke`；所有指令都經過 `noteRepository` / `windowService` / `shortcutService`。
- 每次寫入後 Rust 端會廣播 `notes:changed` / `notes:removed`，所有視窗的 store 透過 `noteRepository.subscribe` 保持同步。
- Note 視窗由 Rust 建立（label `note-{id}`），同一張便籤永遠只會有一個視窗；搭配 single-instance plugin 防止多開。
- Tray 與 `Ctrl+Alt+N` 註冊在 Rust 端，即使管理視窗沒開也能用；快捷鍵註冊失敗只會在管理視窗底部顯示提示，不會 crash。

### 前端 `src/`

| 路徑 | 職責 |
| --- | --- |
| `views/MainView.vue` | 便籤管理（全部 / 封存 / 垃圾桶、搜尋、標籤篩選、排序與拖曳、多選批次操作） |
| `views/SearchView.vue` | 快速搜尋視窗（`Ctrl+Alt+F`，label `search`，隱藏而不關閉） |
| `views/NoteView.vue` | 單張便籤視窗 |
| `components/note/*` | 便籤 Header、選單（顏色、封存、刪除） |
| `components/main/*` | 清單項目 |
| `components/tags/*` + `utils/tags.ts` | 標籤 chip、輸入（自動補完）、篩選列、管理對話框；正規化規則與 Rust `models/note.rs` 一致 |
| `components/ui/*` | shadcn-vue 風格基礎元件（Reka UI） |
| `composables/useNoteDraft.ts` | 標題 / 內容草稿 + 400ms debounce 自動儲存 |
| `composables/useNoteWindow.ts` | 位置尺寸保存（300ms debounce）、置頂、收合、顏色 |
| `stores/note.ts` | Pinia store |
| `repositories/noteRepository.ts` | Note commands 與同步事件 |
| `services/windowService.ts` | 視窗開關、收合、幾何監聽 |
| `constants/noteColors.ts` + `styles/note-colors.css` | 顏色 → CSS 變數 |

### Rust `src-tauri/`

| 路徑 | 職責 |
| --- | --- |
| `migrations/*.sql` | 版本化 schema migration（記錄在 `schema_migrations`） |
| `src/database/` | 連線、migration runner、`note_repository`（SQL） |
| `src/models/` | `Note`、`NoteUpdate` |
| `src/commands/` | Tauri commands（notes / windows / app） |
| `src/services/` | 業務流程：notes、window、lifecycle（啟動 / 關閉）、tray、shortcut |

## 行為說明

- **關閉便籤視窗（×）不是刪除**：只會記錄 `isOpen = false`，下次啟動不再自動開啟，可從管理視窗點擊重新開啟。從系統匣「結束」App 則保留所有開著的便籤，下次原樣還原。
- **位置**存實體像素（多螢幕安全），**尺寸**存邏輯像素（DPI 無關）；若原本的螢幕已拔除，便籤會改置中顯示。
- **收合**只把視窗縮成標題列，`height` 永遠保存展開時的高度，展開即還原。
- `updatedAt` 只在標題、內容、顏色變更時更新；拖曳、縮放、置頂、收合不會影響清單排序。
- 刪除採 soft delete（`status = 'deleted'`、`deletedAt`），可從垃圾桶恢復或永久刪除。垃圾桶中超過保留天數（預設 30 天，可設 7 / 30 / 90 天或永不，存於 `settings.trash_retention_days`）的便籤會在啟動時與之後每小時自動永久刪除；垃圾桶分頁也可一鍵清空。
- **排序**：管理清單可依最後修改、建立時間、標題或手動排序；手動排序以拖曳調整，存在 `notes.sort_order`（migration 0005），新便籤排在最前面。排序方式與標籤篩選記在該視窗的 localStorage。
- **批次操作**：打開清單上方的「批次操作」開關後，點擊便籤改為選取（Shift+點擊選取範圍、Ctrl+A 全選，Esc 關閉開關）。開啟時底部改為批次操作列：顏色、標籤（多張便籤共用的編輯對話框）、封存 / 取消封存、移到垃圾桶、恢復、永久刪除。
- **整理桌面便籤**（設定選單的子選單）：先選排列方式「並排」（保留各自大小，排滿後錯開重來）或「疊放」，再選要放在主螢幕工作區的左上 / 右上 / 左下 / 右下角或畫面中央；靠右或靠下時從該角落往內排。排列方式與上次的位置記在 localStorage，新位置由各便籤視窗的位置監聽自動存檔。
- **依標籤顯示**：在「全部」分頁用標籤篩選後按「桌面只顯示這些」，桌面上只留下篩選出的便籤，其他便籤會關閉（`isOpen = false`），可用「顯示全部便籤」叫回。
- **便籤連結**：在內文輸入 `[[` 會列出其他便籤（可輸入文字搜尋、↑↓ 選擇、Enter / Tab 插入；找不到時可直接新增一張同名便籤）。連結存成 `[標題](pinnote://note/<id>)`，**Ctrl+點擊** 開啟該便籤（封存的會先取消封存；在垃圾桶或已永久刪除時會提示）。連結文字是插入當下的標題，之後改名不會自動更新。
- **歷史版本**（便籤選單）：編輯標題或內容時，若距離上次保留超過 10 分鐘，會把修改前的內容存進 `note_revisions`（migration 0006），每張便籤最多 50 個版本，永久刪除便籤時一併刪除。可預覽並還原任一版本，還原前的內容也會保留成一個版本。舊版本引用的圖片不會被自動清理。
- **跨裝置同步**（設定選單 →「跨裝置同步…」）：選一個雲端硬碟（Google 雲端硬碟、OneDrive…）裡的資料夾，資料放在其中的 `PinNote Sync/`：`notes/<id>.json`（一張便籤一個檔）、`deleted/<id>.json`（永久刪除紀錄，保留 90 天）、`images/`、`settings.json`（標籤顏色）。同步標題、內容、顏色、標籤、狀態、手動排序與標籤顏色；視窗位置大小、置頂、收合、是否開在桌面、快捷鍵各台分開。
  - 合併：以上次同步的狀態（`sync_base`）做逐欄位三方合併；兩邊都改了同一欄位時採用 `modified_at`（migration 0007，任何同步欄位變動就更新）較新的一方，**輸掉的文字存進該便籤的歷史版本**並提示。標籤以集合合併。刪除只會蓋過刪除時間之前的修改。
  - 雲端硬碟產生的衝突副本（`<id> (1).json`、`<id>-電腦名稱.json`）會當成另一個版本合併後刪除；寫檔先寫暫存檔再改名。圖片只會補齊、不會從同步資料夾刪除。
  - 同步資料夾不見時（曾經同步過、但資料夾不存在或裡面沒有任何便籤與刪除紀錄）**不會自動重建**：同步暫停，管理視窗顯示提示，可選「重新建立並上傳這台電腦的便籤」、「選擇其他資料夾」或「停止同步」；資料夾回來（例如雲端硬碟程式重新啟動）會自動繼續。換到另一個資料夾時會清掉合併基準，像第一次同步一樣合併，不會刪除任何便籤。
  - 時機：啟動時、本機變更約 20 秒後、每 2 分鐘，以及「立即同步」。在背景執行緒執行；剛被修改的便籤會留到下一輪，不會蓋掉正在輸入的內容。開著的便籤收到其他裝置的修改會重新載入；若有尚未存檔的輸入則保留輸入，對方的內容存成歷史版本。
- **標籤**：獨立欄位 `notes.tags`（JSON 陣列，migration 0004），不分大小寫、最長 30 字。標籤顏色存在 `settings.tag_colors`。
- **Markdown 編輯**（Tiptap）：所見即所得，內容以 Markdown 存在 `content` 欄位，舊的純文字便籤直接相容（單一換行視為換行）。支援 Markdown 快速輸入（`# `、`- `、`1. `、`[ ] `、`> `、```` ``` ````、`**粗體**`、`==標記==`…）與底部格式列（hover 或編輯時顯示）。底線存為 `++文字++`、螢光標記存為 `==文字==`。連結需 **Ctrl+點擊** 才會用系統瀏覽器開啟（只允許 http / https / mailto）。設定在 `src/lib/noteEditor.ts`。
  - **表格**：GFM 表格（含欄位對齊），工具列可插入表格，游標在表格內時可新增 / 刪除列與欄。
  - **圖片**：可貼上截圖、從檔案總管拖入或用工具列選檔（png / jpg / gif / webp / bmp，單張 ≤ 20 MB）。檔案存於 `%APPDATA%\com.pinnote.app\images\`，Markdown 記錄為 `![alt](images/<檔名>)`，透過 asset protocol 顯示；網路圖片 `![](https://…)` 直接顯示。永久刪除便籤時會一併刪除只有它使用的圖片；啟動時與「設定 → 清理未使用的圖片」會刪除沒有任何便籤（含垃圾桶）引用、且超過 24 小時的圖片。
  - **防止資料遺失**：載入時若偵測到編輯器無法呈現的內容（例如原始 HTML），便籤會切為唯讀並提示，可「複製原始內容」或確認後「仍要編輯」。
- **全域快捷鍵**（Rust 端註冊，可在「設定 → 自訂快捷鍵」錄製、停用或恢復預設；存於 `settings` 表 `shortcut.<action>`）。預設：
  - `Ctrl+Alt+N` 新增便籤
  - `Ctrl+Alt+V` 以剪貼簿文字建立便籤（剪貼簿沒有文字時建立空白便籤）
  - `Ctrl+Alt+H` 暫時隱藏 / 顯示所有便籤視窗（不改變 `isOpen`，重開 App 一律顯示）
  - `Ctrl+Alt+F` 快速搜尋（再按一次或失去焦點時隱藏）：搜尋進行中與封存的便籤，支援 `#標籤`；Enter 開啟（封存的會先取消封存），Ctrl+Enter 以輸入文字為標題新增便籤

  錄製時會暫停所有全域快捷鍵。單鍵只接受 F1–F24（適合鍵盤巨集鍵），其他按鍵需搭配 Ctrl / Alt / Shift / Win；與其他動作重複或被其他程式占用時會保留原設定。
- **置頂便籤半透明**：置頂便籤在沒有 focus、滑鼠也不在上面時淡出，可在設定選單關閉。便籤視窗因此是透明的原生視窗；偏好設定存在 SQLite `settings` 表（migration 0003），變更時廣播 `settings:changed`。
- 管理視窗清單會標示「未顯示」（已用 × 關閉）的便籤；設定選單與系統匣都有「顯示全部便籤」。
- **開機自動啟動**（設定選單）：開機啟動時帶 `--autostart`，只還原桌面便籤，不開管理視窗。
- **自動備份**：每天第一次啟動、執行 migration 之前，以 `VACUUM INTO` 備份到 `%APPDATA%\com.pinnote.app\backups\pinnote-YYYYMMDD.db`，保留最近 7 份。這些快照引用到的圖片會同步到 `backups\images\`（只複製新檔，不再被任何快照引用的會移除）。還原方式：結束 App，用備份檔覆蓋 `pinnote.db`，並把 `backups\images\` 的檔案複製回 `images\`。
- **自動更新**：啟動 30 秒後與之後每 24 小時，向 GitHub 最新一個「已公開」Release 的 `latest.json` 檢查新版本（開發模式不會自動檢查）。有新版時管理視窗顯示提示、系統匣多一項「更新到 vX…」；按「稍後」的版本不再提示。更新檔以 minisign 簽章，公鑰在 `tauri.conf.json` 的 `plugins.updater.pubkey`，驗證通過才安裝；安裝前會先儲存所有便籤，安裝後自動重新啟動。
- **匯出 / 匯入 JSON**（設定選單）：匯出包含所有狀態的便籤，以及它們使用的圖片（base64 內嵌，格式 v3 含標籤；仍可匯入舊的 v1 / v2 檔）；匯入時 id 已存在的便籤會略過，所以重複匯入同一個檔案不會產生重複便籤。匯入的便籤不會自動開到桌面。

## 發佈新版

1. 把 `package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml` 的版本號改成新版，合併到 `main` 並 push。
2. [release.yml](.github/workflows/release.yml) 發現這個版本還沒有 tag / Release，就會測試、打包，建立草稿 Release（含安裝檔與自動更新用的 `latest.json`）。
3. 到 GitHub Releases 補上更新說明後按 **Publish**。公開後，已安裝的 App 才會收到更新（草稿不會被偵測到）。

更新檔簽章用的私鑰與密碼存在 repo 的 Actions secrets：`TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。**私鑰遺失就無法再推送更新**，請另外備份。

## 擴充方式

- 新欄位：新增 `src-tauri/migrations/000N_*.sql` 並加入 `MIGRATIONS` 清單，再擴充 `Note` / `NoteUpdate`；若要跨裝置同步，也要加進 `services/sync` 的同步欄位與合併規則。
- Reminder：在 `services/` 新增排程服務，透過事件通知對應視窗。

## License

[MIT](LICENSE)

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- Committers and reviewers: [Cheng-Ching-Wu](https://github.com/Cheng-Ching-Wu)
- Approvers: [Cheng-Ching-Wu](https://github.com/Cheng-Ching-Wu)

Release binaries are built from this repository by GitHub Actions ([release.yml](.github/workflows/release.yml)) on GitHub-hosted runners.

### Privacy policy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it, with one exception: to check for updates it downloads a small version file (`latest.json`) from this repository's GitHub Releases at startup and once a day. This request sends no notes or personal data, and can be turned off in Settings → "Check for updates automatically". Notes are stored locally in SQLite; cross-device sync (off by default) only writes to a folder the user chooses, such as one inside a cloud drive client.
