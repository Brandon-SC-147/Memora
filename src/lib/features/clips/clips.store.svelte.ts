import { clipsApi } from "$lib/api";
import type { ClipboardEntry } from "$lib/api";

/** Filtros de la vista actual de clips. */
export interface ClipsFilter {
  workspaceId: string | null;
  favoriteOnly: boolean;
  pinnedOnly: boolean;
}

export const defaultClipsFilter: ClipsFilter = {
  workspaceId: null,
  favoriteOnly: false,
  pinnedOnly: false,
};

/**
 * Estado global de la lista de clips (feature clips).
 *
 * La lista se recarga cuando cambian los filtros; las operaciones
 * puntuales (favorito/pin) mutan en memoria sin re-fetchear.
 */
export class ClipsStore {
  items = $state<ClipboardEntry[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  filter = $state<ClipsFilter>({ ...defaultClipsFilter });

  async load(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.items = await clipsApi.listClips({
        workspaceId: this.filter.workspaceId,
        favoriteOnly: this.filter.favoriteOnly,
        pinnedOnly: this.filter.pinnedOnly,
        limit: 100,
      });
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  setFilter(patch: Partial<ClipsFilter>): void {
    this.filter = { ...this.filter, ...patch };
    void this.load();
  }

  toggleFavorite(id: string): void {
    this.items = this.items.map((clip) =>
      clip.id === id ? { ...clip, favorite: !clip.favorite } : clip,
    );
  }

  togglePinned(id: string): void {
    this.items = this.items.map((clip) =>
      clip.id === id ? { ...clip, pinned: !clip.pinned } : clip,
    );
  }
}

export const clips = new ClipsStore();