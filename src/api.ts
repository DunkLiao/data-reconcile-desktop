import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import type {
  CompareOptions,
  CompareResult,
  DelimiterOption,
  EncodingOption,
  FileInfo,
  ProgressEvent,
} from "./types";

export async function pickFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: [
      { name: "CSV Files", extensions: ["csv"] },
      { name: "Text Files", extensions: ["txt"] },
      { name: "All Files", extensions: ["*"] },
    ],
  });
  if (typeof selected === "string") {
    return selected;
  }
  return null;
}

export async function inspectFile(
  path: string,
  encoding: EncodingOption,
  delimiter: DelimiterOption
): Promise<FileInfo> {
  return await invoke<FileInfo>("inspect_file", { path, encoding, delimiter });
}

export async function compareFiles(options: CompareOptions): Promise<CompareResult> {
  return await invoke<CompareResult>("compare_files", { options });
}

export async function exportExcel(result: CompareResult, outputPath: string): Promise<string> {
  return await invoke<string>("export_excel", { result, outputPath });
}

export async function cancelCompare(): Promise<void> {
  await invoke("cancel_compare");
}

export async function onProgress(callback: (e: ProgressEvent) => void): Promise<() => void> {
  return await listen<ProgressEvent>("compare-progress", (event) => callback(event.payload));
}