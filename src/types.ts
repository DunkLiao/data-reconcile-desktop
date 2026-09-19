export type EncodingOption = "auto" | "utf8" | "big5" | "cp950" | "utf16le" | "utf16be";

export type DelimiterType = "auto" | "comma" | "semicolon" | "tab" | "pipe" | "colon" | "custom";

export interface DelimiterOption {
  type: DelimiterType;
  value?: string;
}

export interface ParseOptions {
  encoding: EncodingOption;
  delimiter: DelimiterOption;
}

export interface FileInfo {
  path: string;
  encoding: string;
  encoding_confident: boolean;
  encoding_warning: string | null;
  delimiter: string;
  delimiter_confident: boolean;
  headers: string[];
  row_count: number | null;
}

export type ComparisonMode = "key_based" | "row_by_row";

export interface CompareOptions {
  file_a_path: string;
  file_b_path: string;
  file_a_parse_options: ParseOptions;
  file_b_parse_options: ParseOptions;
  comparison_mode: ComparisonMode;
  key_columns: string[];
  excluded_columns: string[];
  trim_whitespace: boolean;
  ignore_case: boolean;
}

export type DifferenceType =
  | "VALUE_CHANGED"
  | "A_ONLY"
  | "B_ONLY"
  | "COLUMN_A_ONLY"
  | "COLUMN_B_ONLY"
  | "DUPLICATE_KEY_A"
  | "DUPLICATE_KEY_B"
  | "EMPTY_KEY_A"
  | "EMPTY_KEY_B"
  | "INCOMPLETE_KEY_A"
  | "INCOMPLETE_KEY_B";

export interface KeyValue {
  column: string;
  value: string;
}

export interface Difference {
  key_values: KeyValue[];
  row_a: number | null;
  row_b: number | null;
  column_name: string | null;
  value_a: string | null;
  value_b: string | null;
  difference_type: DifferenceType;
}

export interface DuplicateKeyInfo {
  source: string;
  key_values: KeyValue[];
  count: number;
  rows: number[];
}

export interface CompareResult {
  identical: boolean;
  rows_a: number;
  rows_b: number;
  key_columns: string[];
  compared_columns: string[];
  excluded_columns: string[];
  column_a_only: string[];
  column_b_only: string[];
  matched_records: number;
  same_records: number;
  different_records: number;
  a_only_records: number;
  b_only_records: number;
  duplicate_keys_a: number;
  duplicate_keys_b: number;
  empty_keys_a: number;
  empty_keys_b: number;
  incomplete_keys_a: number;
  incomplete_keys_b: number;
  different_cells: number;
  differences: Difference[];
  duplicate_details_a: DuplicateKeyInfo[];
  duplicate_details_b: DuplicateKeyInfo[];
  file_a_name: string;
  file_b_name: string;
  file_a_encoding: string;
  file_b_encoding: string;
  file_a_delimiter: string;
  file_b_delimiter: string;
  compare_mode: string;
  compare_time: string;
  cancelled: boolean;
}

export interface ProgressEvent {
  processed: number;
  total: number | null;
  stage: string;
}

export const ENCODING_OPTIONS: { value: EncodingOption; label: string }[] = [
  { value: "auto", label: "Auto" },
  { value: "utf8", label: "UTF-8" },
  { value: "big5", label: "Big5" },
  { value: "cp950", label: "CP950" },
  { value: "utf16le", label: "UTF-16 LE" },
  { value: "utf16be", label: "UTF-16 BE" },
];

export const DELIMITER_OPTIONS: { value: DelimiterType; label: string }[] = [
  { value: "auto", label: "Auto" },
  { value: "comma", label: "Comma (,)" },
  { value: "semicolon", label: "Semicolon (;)" },
  { value: "tab", label: "Tab" },
  { value: "pipe", label: "Pipe (|)" },
  { value: "colon", label: "Colon (:)" },
  { value: "custom", label: "Custom" },
];

export const TYPE_LABELS: Record<DifferenceType, string> = {
  VALUE_CHANGED: "Value Changed",
  A_ONLY: "A Only",
  B_ONLY: "B Only",
  COLUMN_A_ONLY: "Column A Only",
  COLUMN_B_ONLY: "Column B Only",
  DUPLICATE_KEY_A: "Duplicate Key A",
  DUPLICATE_KEY_B: "Duplicate Key B",
  EMPTY_KEY_A: "Empty Key A",
  EMPTY_KEY_B: "Empty Key B",
  INCOMPLETE_KEY_A: "Incomplete Key A",
  INCOMPLETE_KEY_B: "Incomplete Key B",
};