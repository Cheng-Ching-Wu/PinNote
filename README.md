# PinNote

輕量的 Windows 桌面便籤。每張便籤都是獨立的 Tauri 視窗，內容、位置、尺寸、顏色、置頂與收合狀態都會自動存進 SQLite，重開後原樣還原。

## 開發

需求：Node 20+、Rust stable、Visual Studio Build Tools（C++ workload + Windows SDK）、WebView2。

```bash
npm install
npm run tauri dev      # 開發模式
npm run tauri build    # 打包安裝檔
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
| `views/MainView.vue` | 便籤管理（全部 / 封存 / 垃圾桶、搜尋） |
| `views/NoteView.vue` | 單張便籤視窗 |
| `components/note/*` | 便籤 Header、選單（顏色、封存、刪除） |
| `components/main/*` | 清單項目 |
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
- 刪除採 soft delete（`status = 'deleted'`、`deletedAt`），可從垃圾桶恢復或永久刪除。
- **Markdown 編輯**（Tiptap）：所見即所得，內容以 Markdown 存在 `content` 欄位，舊的純文字便籤直接相容（單一換行視為換行）。支援 Markdown 快速輸入（`# `、`- `、`1. `、`[ ] `、`> `、```` ``` ````、`**粗體**`、`==標記==`…）與底部格式列（hover 或編輯時顯示）。底線存為 `++文字++`、螢光標記存為 `==文字==`。連結需 **Ctrl+點擊** 才會用系統瀏覽器開啟（只允許 http / https / mailto）。設定在 `src/lib/noteEditor.ts`。
  - **表格**：GFM 表格（含欄位對齊），工具列可插入表格，游標在表格內時可新增 / 刪除列與欄。
  - **圖片**：可貼上截圖、從檔案總管拖入或用工具列選檔（png / jpg / gif / webp / bmp，單張 ≤ 20 MB）。檔案存於 `%APPDATA%\com.pinnote.app\images\`，Markdown 記錄為 `![alt](images/<檔名>)`，透過 asset protocol 顯示；網路圖片 `![](https://…)` 直接顯示。永久刪除便籤時會一併刪除只有它使用的圖片；啟動時與「設定 → 清理未使用的圖片」會刪除沒有任何便籤（含垃圾桶）引用、且超過 24 小時的圖片。
  - **防止資料遺失**：載入時若偵測到編輯器無法呈現的內容（例如原始 HTML），便籤會切為唯讀並提示，可「複製原始內容」或確認後「仍要編輯」。
- **全域快捷鍵**（Rust 端註冊，可在「設定 → 自訂快捷鍵」錄製、停用或恢復預設；存於 `settings` 表 `shortcut.<action>`）。預設：
  - `Ctrl+Alt+N` 新增便籤
  - `Ctrl+Alt+V` 以剪貼簿文字建立便籤（剪貼簿沒有文字時建立空白便籤）
  - `Ctrl+Alt+H` 暫時隱藏 / 顯示所有便籤視窗（不改變 `isOpen`，重開 App 一律顯示）

  錄製時會暫停所有全域快捷鍵。單鍵只接受 F1–F24（適合鍵盤巨集鍵），其他按鍵需搭配 Ctrl / Alt / Shift / Win；與其他動作重複或被其他程式占用時會保留原設定。
- **置頂便籤半透明**：置頂便籤在沒有 focus、滑鼠也不在上面時淡出，可在設定選單關閉。便籤視窗因此是透明的原生視窗；偏好設定存在 SQLite `settings` 表（migration 0003），變更時廣播 `settings:changed`。
- 管理視窗清單會標示「未顯示」（已用 × 關閉）的便籤；設定選單與系統匣都有「顯示全部便籤」。
- **開機自動啟動**（設定選單）：開機啟動時帶 `--autostart`，只還原桌面便籤，不開管理視窗。
- **自動備份**：每天第一次啟動、執行 migration 之前，以 `VACUUM INTO` 備份到 `%APPDATA%\com.pinnote.app\backups\pinnote-YYYYMMDD.db`，保留最近 7 份。這些快照引用到的圖片會同步到 `backups\images\`（只複製新檔，不再被任何快照引用的會移除）。還原方式：結束 App，用備份檔覆蓋 `pinnote.db`，並把 `backups\images\` 的檔案複製回 `images\`。
- **匯出 / 匯入 JSON**（設定選單）：匯出包含所有狀態的便籤，以及它們使用的圖片（base64 內嵌，格式 v2；仍可匯入舊的 v1 檔）；匯入時 id 已存在的便籤會略過，所以重複匯入同一個檔案不會產生重複便籤。匯入的便籤不會自動開到桌面。

## 後續擴充預留

- 新欄位：新增 `src-tauri/migrations/000N_*.sql` 並加入 `MIGRATIONS` 清單，再擴充 `Note` / `NoteUpdate`。
- Checklist / Markdown Lite：替換 `NoteView` 中的 textarea 為獨立編輯元件即可，儲存流程（`useNoteDraft`）不變。
- Tag：新增 `tags`、`note_tags` 表與對應 repository / store。
- Reminder：在 `services/` 新增排程服務，透過事件通知對應視窗。
- Cloud Sync：`noteRepository` 是唯一資料出入口，可在其後加入同步層。
