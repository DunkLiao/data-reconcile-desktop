import { useEffect } from "react";

import { inspectFile, pickFile } from "./api";
import type { DelimiterOption, EncodingOption, FileInfo } from "./types";
import { DELIMITER_OPTIONS, ENCODING_OPTIONS } from "./types";

export interface FilePanelState {
  path: string;
  encoding: EncodingOption;
  delimiterType: DelimiterOption["type"];
  customDelimiter: string;
  info: FileInfo | null;
  inspecting: boolean;
  error: string | null;
}

export function makeFilePanel(): FilePanelState {
  return {
    path: "",
    encoding: "auto",
    delimiterType: "auto",
    customDelimiter: "",
    info: null,
    inspecting: false,
    error: null,
  };
}

export function formatNumber(n: number): string {
  return n.toLocaleString("en-US");
}

export function isValidCustomDelimiter(panel: FilePanelState): boolean {
  if (panel.delimiterType !== "custom") return true;
  return panel.customDelimiter.length === 1 && panel.customDelimiter.charCodeAt(0) <= 0x7f;
}

export function FilePanel({
  label,
  panel,
  onChange,
}: {
  label: string;
  panel: FilePanelState;
  onChange: (p: FilePanelState) => void;
}) {
  const update = (patch: Partial<FilePanelState>) => onChange({ ...panel, ...patch });

  const browse = async () => {
    const path = await pickFile();
    if (path) update({ path, info: null, error: null });
  };

  useEffect(() => {
    if (!panel.path) {
      update({ info: null, error: null });
      return;
    }
    if (!isValidCustomDelimiter(panel)) {
      update({ info: null, inspecting: false, error: "自訂分隔符必須是單一 ASCII 字元。" });
      return;
    }
    let cancelled = false;
    update({ inspecting: true, error: null });
    const delimiterOption: DelimiterOption =
      panel.delimiterType === "custom"
        ? { type: "custom", value: panel.customDelimiter }
        : { type: panel.delimiterType };
    inspectFile(panel.path, panel.encoding, delimiterOption)
      .then((info) => {
        if (!cancelled) update({ info, inspecting: false, error: null });
      })
      .catch((e: unknown) => {
        const msg = typeof e === "string" ? e : String(e);
        if (!cancelled) update({ info: null, inspecting: false, error: msg });
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [panel.path, panel.encoding, panel.delimiterType, panel.customDelimiter]);

  return (
    <div className="file-panel">
      <div className="file-title">{label}</div>
      <div className="row path-row">
        <input
          className="path-input"
          value={panel.path}
          placeholder={`請輸入 ${label} 檔案路徑`}
          onChange={(e) => update({ path: e.target.value, info: null, error: null })}
          spellCheck={false}
        />
        <button className="btn" onClick={browse}>
          瀏覽
        </button>
      </div>
      <div className="row selects">
        <label>
          Encoding
          <select
            value={panel.encoding}
            onChange={(e) => update({ encoding: e.target.value as EncodingOption })}
          >
            {ENCODING_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          Delimiter
          <select
            value={panel.delimiterType}
            onChange={(e) =>
              update({ delimiterType: e.target.value as DelimiterOption["type"] })
            }
          >
            {DELIMITER_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
        </label>
        {panel.delimiterType === "custom" && (
          <label>
            自訂分隔符
            <input
              className="custom-delim"
              maxLength={1}
              value={panel.customDelimiter}
              onChange={(e) => update({ customDelimiter: e.target.value })}
            />
          </label>
        )}
      </div>
      {panel.inspecting && <div className="hint">正在分析檔案...</div>}
      {panel.info && !panel.error && (
        <div className="hint">
          偵測結果：Encoding <b>{panel.info.encoding}</b> ｜ Delimiter{" "}
          <b>{panel.info.delimiter}</b> ｜ Columns <b>{panel.info.headers.length}</b>
          {panel.info.encoding_warning && (
            <div className="warn">{panel.info.encoding_warning}</div>
          )}
        </div>
      )}
      {panel.error && <div className="warn">{panel.error}</div>}
    </div>
  );
}

export function CheckboxList({
  columns,
  selected,
  disabled,
  search,
  onToggle,
  onSearch,
  showExtraButtons,
  onSelectAll,
  onClearAll,
  onInvert,
}: {
  columns: string[];
  selected: Set<string>;
  disabled: (col: string) => boolean;
  search: string;
  onToggle: (col: string) => void;
  onSearch: (s: string) => void;
  showExtraButtons: boolean;
  onSelectAll: () => void;
  onClearAll: () => void;
  onInvert: () => void;
}) {
  const filtered = columns.filter((c) => c.toLowerCase().includes(search.toLowerCase()));
  return (
    <div className="checkbox-panel">
      <div className="row">
        <input
          className="search-input"
          placeholder="搜尋欄位"
          value={search}
          onChange={(e) => onSearch(e.target.value)}
        />
      </div>
      {showExtraButtons && (
        <div className="row btn-row">
          <button className="mini-btn" onClick={onSelectAll}>
            全選
          </button>
          <button className="mini-btn" onClick={onClearAll}>
            全部取消
          </button>
          <button className="mini-btn" onClick={onInvert}>
            反向選取
          </button>
        </div>
      )}
      <div className="checkbox-list">
        {filtered.length === 0 && <div className="hint">無符合欄位</div>}
        {filtered.map((c) => (
          <label key={c} className={disabled(c) ? "checkbox disabled" : "checkbox"}>
            <input
              type="checkbox"
              checked={selected.has(c)}
              disabled={disabled(c)}
              onChange={() => onToggle(c)}
            />
            <span>{c}</span>
          </label>
        ))}
      </div>
    </div>
  );
}
