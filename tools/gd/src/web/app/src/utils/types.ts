// Types matching the Rust protocol (src/web/protocol.rs)

export type WebLineKind = "context" | "added" | "deleted";
export type WebFileStatus =
  | "modified"
  | "added"
  | "deleted"
  | "renamed"
  | "untracked";

export interface WebDiffLine {
  kind: WebLineKind;
  content_html: string;
  raw_content: string;
  old_lineno: number | null;
  new_lineno: number | null;
  line_idx: number;
}

export interface WebDiffHunk {
  old_start: number;
  new_start: number;
  lines: WebDiffLine[];
}

export interface WebDiffFile {
  path: string;
  old_path: string | null;
  status: WebFileStatus;
  hunks: WebDiffHunk[];
}

export interface WebTreeEntry {
  label: string;
  depth: number;
  file_idx: number | null;
  status: WebFileStatus | null;
  is_dir: boolean;
  collapsed: boolean;
  icon: string;
  icon_color: string;
}

export interface DiffData {
  type: "DiffData";
  files: WebDiffFile[];
  tree: WebTreeEntry[];
  branch: string;
  source_label: string;
}

export interface ClientMessage {
  type: "SetFullContext";
  enabled: boolean;
}

export type ServerMessage = DiffData;

export function isDiffData(msg: unknown): msg is DiffData {
  return (
    typeof msg === "object" &&
    msg !== null &&
    "type" in msg &&
    (msg as Record<string, unknown>).type === "DiffData"
  );
}
