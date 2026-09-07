import { clipsApi } from "$lib/api";
import type { SearchResult } from "$lib/api";

/**
 * Estado global de la búsqueda FTS5 (feature search).
 *
 * La búsqueda se dispara al cambiar `query` (debounce desde la UI);
 * los resultados mantienen el ranking BM25 del backend.
 */
export class SearchStore {
  query = $state("");
  results = $state<SearchResult[]>([]);
  active = $state(false);
  loading = $state(false);
  error = $state<string | null>(null);

  async search(text: string, limit = 50): Promise<void> {
    this.query = text;
    const term = text.trim();
    if (!term) {
      this.results = [];
      this.active = false;
      return;
    }
    this.loading = true;
    this.error = null;
    try {
      this.results = await clipsApi.searchClips({ query: term, limit });
      this.active = true;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  reset(): void {
    this.query = "";
    this.results = [];
    this.active = false;
  }
}

export const search = new SearchStore();