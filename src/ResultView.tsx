import { useMemo, useState } from "react";

import type { CompareResult, Difference, DifferenceType } from "./types";
import { TYPE_LABELS } from "./types";
import { formatNumber } from "./components";

export type FilterKind =
  | "all"
  | "value_changed"
  | "a_only"
  | "b_only"
  | "column"
  | "duplicate"
  | "key_error";

const FILTERS: { value: FilterKind; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "value_changed", label: "Value Changed" },
  { value: "a_only", label: "A Only" },
  { value: "b_only", label: "B Only" },
  { value: "column", label: "Column Difference" },
  { value: "duplicate", label: "Duplicate Key" },
  { value: "key_error", label: "Key Error" },
];

function matchesFilter(t: DifferenceType, f: FilterKind): boolean {
  switch (f) {
    case "all":
      return true;
    case "value_changed":
      return t === "VALUE_CHANGED";
    case "a_only":
      return t === "A_ONLY";
    case "b_only":
      return t === "B_ONLY";
    case "column":
      return t === "COLUMN_A_ONLY" || t === "COLUMN_B_ONLY";
    case "duplicate":
      return t === "DUPLICATE_KEY_A" || t === "DUPLICATE_KEY_B";
    case "key_error":
      return (
        t === "EMPTY_KEY_A" ||
        t === "EMPTY_KEY_B" ||
        t === "INCOMPLETE_KEY_A" ||
        t === "INCOMPLETE_KEY_B"
      );
  }
}

function keyDisplay(d: Difference): string {
  if (d.key_values.length === 0) return "";
  return d.key_values.map((kv) => `${kv.column}=${kv.value}`).join("\n");
}

function typeKey(t: DifferenceType): string {
  if (t === "VALUE_CHANGED") return "changed";
  if (t === "A_ONLY") return "aonly";
  if (t === "B_ONLY") return "bonly";
  if (t === "COLUMN_A_ONLY" || t === "COLUMN_B_ONLY") return "column";
  if (t === "DUPLICATE_KEY_A" || t === "DUPLICATE_KEY_B") return "duplicate";
  return "keyerror";
}

function typeClass(t: DifferenceType): string {
  return `row-${typeKey(t)}`;
}

export function ResultView({
  result,
  onRecompare,
  onClear,
  onExport,
  exporting,
}: {
  result: CompareResult;
  onRecompare: () => void;
  onClear: () => void;
  onExport: () => void;
  exporting: boolean;
}) {
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<FilterKind>("all");

  const rows = useMemo<Difference[]>(() => {
    const duplicateRows = [
      ...result.duplicate_details_a.map((detail) => ({ detail, side: "A" as const })),
      ...result.duplicate_details_b.map((detail) => ({ detail, side: "B" as const })),
    ].map(({ detail, side }): Difference => {
      const description = `${detail.source} | Count: ${detail.count} | Rows: ${detail.rows.join(", ")}`;
      return {
        key_values: detail.key_values,
        row_a: null,
        row_b: null,
        column_name: `Rows: ${detail.rows.join(", ")}`,
        value_a: side === "A" ? description : null,
        value_b: side === "B" ? description : null,
        difference_type: side === "A" ? "DUPLICATE_KEY_A" : "DUPLICATE_KEY_B",
      };
    });
    return [...result.differences, ...duplicateRows];
  }, [result]);

  const filtered = useMemo(() => {
    const q = search.toLowerCase();
    return rows.filter((d) => {
      if (!matchesFilter(d.difference_type, filter)) return false;
      if (!q) return true;
      const haystack = [
        keyDisplay(d),
        d.row_a?.toString() ?? "",
        d.row_b?.toString() ?? "",
        d.column_name ?? "",
        d.value_a ?? "",
        d.value_b ?? "",
        TYPE_LABELS[d.difference_type],
      ]
        .join(" ")
        .toLowerCase();
      return haystack.includes(q);
    });
  }, [rows, search, filter]);

  const keyCount = result.key_columns.length;

  return (
    <div className="result-view">
      <div className={`result-banner ${result.identical ? "ok" : "diff"}`}>
        {result.identical ? "✅ 兩個檔案內容完全相同" : "❌ 資料內容不同"}
      </div>

      <div className="summary-grid">
        <div className="summary-item">
          <div className="label">Compare Mode</div>
          <div className="value">{result.compare_mode}</div>
        </div>
        {keyCount > 0 && (
          <div className="summary-item">
            <div className="label">Key</div>
            <div className="value">{result.key_columns.join(" + ")}</div>
          </div>
        )}
        {result.numeric_tolerance != null && (
          <div className="summary-item">
            <div className="label">Numeric Tolerance</div>
            <div className="value">{`≤ ${result.numeric_tolerance}`}</div>
          </div>
        )}
        <div className="summary-item">
          <div className="label">Rows A</div>
          <div className="value">{formatNumber(result.rows_a)}</div>
        </div>
        <div className="summary-item">
          <div className="label">Rows B</div>
          <div className="value">{formatNumber(result.rows_b)}</div>
        </div>
        <div className="summary-item">
          <div className="label">Same</div>
          <div className="value">{formatNumber(result.same_records)}</div>
        </div>
        <div className="summary-item">
          <div className="label">Different</div>
          <div className="value">{formatNumber(result.different_records)}</div>
        </div>
        <div className="summary-item">
          <div className="label">A Only</div>
          <div className="value">{formatNumber(result.a_only_records)}</div>
        </div>
        <div className="summary-item">
          <div className="label">B Only</div>
          <div className="value">{formatNumber(result.b_only_records)}</div>
        </div>
        <div className="summary-item">
          <div className="label">Different Cells</div>
          <div className="value">{formatNumber(result.different_cells)}</div>
        </div>
        <div className="summary-item">
          <div className="label">Duplicate A / B</div>
          <div className="value">
            {formatNumber(result.duplicate_keys_a)} / {formatNumber(result.duplicate_keys_b)}
          </div>
        </div>
      </div>

      {(result.column_a_only.length + result.column_b_only.length > 0 ||
        result.empty_keys_a + result.empty_keys_b + result.incomplete_keys_a + result.incomplete_keys_b >
          0) && (
        <div className="column-diff-note">
          欄位差異：A Only＝{result.column_a_only.join(", ") || "無"} ｜ B Only＝
          {result.column_b_only.join(", ") || "無"} ｜ Empty Key＝
          {formatNumber(result.empty_keys_a + result.empty_keys_b)} ｜ Incomplete Key＝
          {formatNumber(result.incomplete_keys_a + result.incomplete_keys_b)}
        </div>
      )}

      <div className="toolbar">
        <input
          className="search-input grow"
          placeholder="搜尋 Key / Row / Column / Value / Type"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <select value={filter} onChange={(e) => setFilter(e.target.value as FilterKind)}>
          {FILTERS.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </select>
        <div className="spacer" />
        <span className="hint">顯示 {formatNumber(filtered.length)} / {formatNumber(rows.length)}</span>
        <button className="btn" onClick={onRecompare}>
          重新比較
        </button>
        <button className="btn danger" onClick={onClear}>
          清除結果
        </button>
        <button className="btn primary" onClick={onExport} disabled={exporting}>
          {exporting ? "匯出中..." : "匯出 Excel"}
        </button>
      </div>

      <div className="table-wrap">
        <table className="diff-table">
          <thead>
            <tr>
              {keyCount > 0
                ? result.key_columns.map((k) => <th key={k}>{k}</th>)
                : <th>Key</th>}
              <th>Row A</th>
              <th>Row B</th>
              <th>Column</th>
              <th>File A</th>
              <th>File B</th>
              <th>Type</th>
            </tr>
          </thead>
          <tbody>
            {filtered.length === 0 && (
              <tr>
                <td className="empty" colSpan={keyCount > 0 ? keyCount + 6 : 7}>
                  沒有符合條件的差異
                </td>
              </tr>
            )}
            {filtered.map((d, i) => (
              <tr key={i} className={typeClass(d.difference_type)}>
                {keyCount > 0
                  ? result.key_columns.map((k) => {
                      const kv = d.key_values.find((x) => x.column === k);
                      return <td key={k}>{kv ? kv.value : ""}</td>;
                    })
                  : <td className="cell-key">{keyDisplay(d) || "—"}</td>}
                <td>{d.row_a ?? ""}</td>
                <td>{d.row_b ?? ""}</td>
                <td>{d.column_name ?? "—"}</td>
                <td>{d.value_a ?? ""}</td>
                <td>{d.value_b ?? ""}</td>
                <td className={`type-${typeKey(d.difference_type)}`}>
                  {TYPE_LABELS[d.difference_type]}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
