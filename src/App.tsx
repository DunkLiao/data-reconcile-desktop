import { useEffect, useMemo, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";

import { cancelCompare, compareFiles, exportExcel, onProgress } from "./api";
import type { CompareOptions, CompareResult, ProgressEvent } from "./types";
import {
  CheckboxList,
  FilePanel,
  isValidCustomDelimiter,
  makeFilePanel,
  type FilePanelState,
} from "./components";
import { ResultView } from "./ResultView";

function App() {
  const [fileA, setFileA] = useState<FilePanelState>(makeFilePanel);
  const [fileB, setFileB] = useState<FilePanelState>(makeFilePanel);
  const [mode, setMode] = useState<"key_based" | "row_by_row">("key_based");
  const [keyColumns, setKeyColumns] = useState<Set<string>>(new Set());
  const [keySearch, setKeySearch] = useState("");
  const [excluded, setExcluded] = useState<Set<string>>(new Set());
  const [excludedSearch, setExcludedSearch] = useState("");
  const [trimWhitespace, setTrimWhitespace] = useState(false);
  const [ignoreCase, setIgnoreCase] = useState(false);
  const [tolerance, setTolerance] = useState("");

  const [comparing, setComparing] = useState(false);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [result, setResult] = useState<CompareResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);

  const commonColumns = useMemo(() => {
    if (!fileA.info || !fileB.info) return [];
    return fileA.info.headers.filter((h) => fileB.info!.headers.includes(h));
  }, [fileA.info, fileB.info]);

  const allColumns = useMemo(() => {
    const set = new Set<string>();
    fileA.info?.headers.forEach((h) => set.add(h));
    fileB.info?.headers.forEach((h) => set.add(h));
    return Array.from(set);
  }, [fileA.info, fileB.info]);

  // Prune selections that are no longer valid (e.g. column disappeared).
  useEffect(() => {
    const validKeys = new Set([...keyColumns].filter((k) => commonColumns.includes(k)));
    if (validKeys.size !== keyColumns.size) setKeyColumns(validKeys);
  }, [commonColumns, keyColumns]);

  useEffect(() => {
    const valid = new Set([...excluded].filter((c) => allColumns.includes(c)));
    if (valid.size !== excluded.size) setExcluded(valid);
  }, [allColumns, excluded]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    onProgress((e) => setProgress(e)).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const toggleSet = (set: Set<string>, value: string): Set<string> => {
    const next = new Set(set);
    if (next.has(value)) next.delete(value);
    else next.add(value);
    return next;
  };

  const buildOptions = (): CompareOptions | null => {
    if (!fileA.path || !fileB.path) {
      setError("請先選擇 File A 與 File B。");
      return null;
    }
    if (!fileA.info || !fileB.info) {
      setError("檔案尚未成功解析，請檢查錯誤訊息。");
      return null;
    }
    if (!isValidCustomDelimiter(fileA) || !isValidCustomDelimiter(fileB)) {
      setError("自訂分隔符必須是單一 ASCII 字元。");
      return null;
    }
    if (
      (fileA.encoding === "auto" && !fileA.info.encoding_confident) ||
      (fileB.encoding === "auto" && !fileB.info.encoding_confident)
    ) {
      setError("Auto Encoding 信心不足，請手動選擇 Encoding 後再比較。");
      return null;
    }
    if (mode === "key_based" && keyColumns.size === 0) {
      setError("請至少選擇一個 Key 欄位，或改用「依資料列順序比對」。");
      return null;
    }
    const trimmedTolerance = tolerance.trim();
    let numericTolerance: number | null = null;
    if (trimmedTolerance !== "") {
      const parsed = Number(trimmedTolerance);
      if (!Number.isFinite(parsed) || parsed < 0) {
        setError("數值誤差容許值必須是大於等於 0 的數字，或留空表示關閉。");
        return null;
      }
      numericTolerance = parsed;
    }
    const makeParse = (p: FilePanelState) => ({
      encoding: p.encoding,
      delimiter:
        p.delimiterType === "custom"
          ? { type: "custom" as const, value: p.customDelimiter }
          : { type: p.delimiterType as "auto" },
    });
    return {
      file_a_path: fileA.path,
      file_b_path: fileB.path,
      file_a_parse_options: makeParse(fileA),
      file_b_parse_options: makeParse(fileB),
      comparison_mode: mode,
      key_columns: mode === "key_based" ? Array.from(keyColumns) : [],
      excluded_columns: Array.from(excluded),
      trim_whitespace: trimWhitespace,
      ignore_case: ignoreCase,
      numeric_tolerance: numericTolerance,
    };
  };

  const startCompare = async () => {
    const options = buildOptions();
    if (!options) return;
    setError(null);
    setResult(null);
    setProgress({ processed: 0, total: null, stage: "準備中..." });
    setComparing(true);
    try {
      const res = await compareFiles(options);
      setResult(res);
    } catch (e: unknown) {
      const msg = typeof e === "string" ? e : String(e);
      setError(msg);
      setProgress(null);
    } finally {
      setComparing(false);
      setProgress(null);
    }
  };

  const handleCancel = async () => {
    await cancelCompare();
  };

  const clearResult = () => {
    setResult(null);
    setError(null);
  };

  const handleExport = async () => {
    if (!result) return;
    const path = await save({
      defaultPath: "CSV_Compare_Result.xlsx",
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
    });
    if (!path) return;
    setExporting(true);
    try {
      await exportExcel(result, path);
    } catch (e: unknown) {
      setError(typeof e === "string" ? e : String(e));
    } finally {
      setExporting(false);
    }
  };

  const encodingUsable = (p: FilePanelState) =>
    Boolean(p.info) && (p.encoding !== "auto" || p.info!.encoding_confident);
  const canCompare =
    Boolean(fileA.path && fileB.path) &&
    encodingUsable(fileA) &&
    encodingUsable(fileB) &&
    isValidCustomDelimiter(fileA) &&
    isValidCustomDelimiter(fileB) &&
    !fileA.inspecting &&
    !fileB.inspecting &&
    !comparing;

  return (
    <div className="app">
      <header className="app-header">
        <h1>CSV Compare</h1>
        <span className="subtitle">資料核對工具</span>
      </header>

      <main className="app-main">
        <section className="config-section">
          <div className="two-col">
            <FilePanel label="File A" panel={fileA} onChange={setFileA} />
            <FilePanel label="File B" panel={fileB} onChange={setFileB} />
          </div>

          <div className="config-grid">
            <div className="config-block">
              <div className="block-title">比對模式</div>
              <label className="radio">
                <input
                  type="radio"
                  checked={mode === "key_based"}
                  onChange={() => setMode("key_based")}
                />
                使用 Key 比對
              </label>
              <label className="radio">
                <input
                  type="radio"
                  checked={mode === "row_by_row"}
                  onChange={() => setMode("row_by_row")}
                />
                依資料列順序比對
              </label>
            </div>

            {mode === "key_based" && (
              <div className="config-block">
                <div className="block-title">Key 欄位（僅共同欄位）</div>
                <CheckboxList
                  columns={commonColumns}
                  selected={keyColumns}
                  disabled={() => false}
                  search={keySearch}
                  onToggle={(c) => setKeyColumns(toggleSet(keyColumns, c))}
                  onSearch={setKeySearch}
                  showExtraButtons={false}
                  onSelectAll={() => {}}
                  onClearAll={() => {}}
                  onInvert={() => {}}
                />
                <div className="composite-key">
                  Composite Key：{keyColumns.size === 0 ? "尚未選擇" : Array.from(keyColumns).join(" + ")}
                </div>
              </div>
            )}

            <div className="config-block">
              <div className="block-title">排除比較欄位</div>
              <CheckboxList
                columns={allColumns}
                selected={excluded}
                disabled={(c) => keyColumns.has(c)}
                search={excludedSearch}
                onToggle={(c) => setExcluded(toggleSet(excluded, c))}
                onSearch={setExcludedSearch}
                showExtraButtons={true}
                onSelectAll={() => setExcluded(new Set(allColumns.filter((c) => !keyColumns.has(c))))}
                onClearAll={() => setExcluded(new Set())}
                onInvert={() =>
                  setExcluded(
                    new Set(allColumns.filter((c) => !keyColumns.has(c) && !excluded.has(c)))
                  )
                }
              />
            </div>

            <div className="config-block">
              <div className="block-title">比較設定</div>
              <label className="radio">
                <input
                  type="checkbox"
                  checked={trimWhitespace}
                  onChange={(e) => setTrimWhitespace(e.target.checked)}
                />
                忽略前後空白
              </label>
              <label className="radio">
                <input
                  type="checkbox"
                  checked={ignoreCase}
                  onChange={(e) => setIgnoreCase(e.target.checked)}
                />
                忽略大小寫
              </label>
              <label className="tolerance-field">
                數值誤差容許值（絕對值，留空＝關閉）
                <input
                  className="tolerance-input"
                  type="number"
                  min="0"
                  step="any"
                  value={tolerance}
                  placeholder="例如 0.01"
                  onChange={(e) => setTolerance(e.target.value)}
                />
              </label>
              <div className="hint">
                僅套用於數值欄位；兩邊皆為數字且絕對差在容許值以內視為相同。Key 欄位不受影響。
              </div>
            </div>
          </div>

          {error && <div className="warn error-box">{error}</div>}

          {comparing ? (
            <div className="progress-box">
              <div className="progress-stage">{progress?.stage ?? "比較中..."}</div>
              <div className="progress-bar">
                <div className="progress-fill indeterminate" />
              </div>
              <div className="progress-count">
                已處理 {progress ? progress.processed.toLocaleString("en-US") : 0} Records
              </div>
              <button className="btn danger" onClick={handleCancel}>
                取消比較
              </button>
            </div>
          ) : (
            <div className="action-row">
              <button className="btn primary big" onClick={startCompare} disabled={!canCompare}>
                開始比較
              </button>
            </div>
          )}
        </section>

        {result && (
          <ResultView
            result={result}
            onRecompare={startCompare}
            onClear={clearResult}
            onExport={handleExport}
            exporting={exporting}
          />
        )}
      </main>
    </div>
  );
}

export default App;
