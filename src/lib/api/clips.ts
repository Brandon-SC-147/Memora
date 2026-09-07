import { transport } from "./invoke";
import type {
  ClipboardEntry,
  ListClipsParams,
  SearchParams,
  SearchResult,
} from "./types";

export function listClips(params?: ListClipsParams): Promise<ClipboardEntry[]> {
  return transport.call("list_clips", {
    params: { ...params } satisfies ListClipsParams,
  });
}

export function searchClips(params: SearchParams): Promise<SearchResult[]> {
  return transport.call("search_clips", { params });
}