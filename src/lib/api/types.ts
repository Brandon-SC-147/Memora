/** Tipos del contrato IPC con el backend Rust (serde camelCase). */

// "text" | "url" | "email" | "json" | "phone" | "color" | "file_path"
// | "command" | "other" | `code:${language}`
export type ContentTypeString = string;

export interface ClipMetadata {
  fields: Record<string, unknown>;
}

/** Espejo de `ClipboardEntry` del dominio Rust. */
export interface ClipboardEntry {
  id: string;
  content: string;
  contentType: ContentTypeString;
  metadata: ClipMetadata;
  contentHash: string;
  isImage: boolean;
  imagePath: string | null;
  copyCount: number;
  favorite: boolean;
  pinned: boolean;
  isSecret: boolean;
  createdAt: string;
  updatedAt: string;
  lastCopiedAt: string;
  workspaceId: string | null;
}

/** Espejo de `Workspace` del dominio Rust. */
export interface Workspace {
  id: string;
  name: string;
  description: string | null;
  icon: string | null;
  createdAt: string;
  isDefault: boolean;
}

/** Resultado de búsqueda FTS5: entrada + score BM25. */
export interface SearchResult {
  entry: ClipboardEntry;
  rank: number;
}

/** `save_clip` → SaveOutcome (camelCase en serde). */
export type SaveOutcome =
  | { created: ClipboardEntry }
  | { duplicate: ClipboardEntry };

/** Info de la app para Diagnostics. */
export interface AppInfo {
  name: string;
  version: string;
}

/** Estado de salud para Diagnostics. */
export interface HealthInfo {
  app: AppInfo;
  databaseHealthy: boolean;
  clipCount: number;
  workspaceCount: number;
}

export interface ListClipsParams {
  workspaceId?: string | null;
  contentType?: string | null;
  favoriteOnly?: boolean;
  pinnedOnly?: boolean;
  limit?: number;
  offset?: number;
}

export interface SearchParams {
  query: string;
  limit?: number;
}