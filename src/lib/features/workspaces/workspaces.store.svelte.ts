import { workspacesApi } from "$lib/api";
import type { Workspace } from "$lib/api";

/**
 * Estado global de workspaces (feature workspaces).
 *
 * Patrón Svelte 5: clase con `$state`, instancia singlete exportada.
 * Las mutaciones del estado son reactivas de forma natural.
 */
export class WorkspacesStore {
  items = $state<Workspace[]>([]);
  active = $state<Workspace | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async load(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const items = await workspacesApi.listWorkspaces();
      this.items = items;
      if (!this.active && items.length > 0) {
        this.active = items[0];
      }
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async create(name: string, description?: string | null): Promise<Workspace | null> {
    try {
      const ws = await workspacesApi.createWorkspace({ name, description });
      this.items = [...this.items, ws];
      return ws;
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  async rename(id: string, name: string): Promise<boolean> {
    try {
      const updated = await workspacesApi.renameWorkspace(id, name);
      this.items = this.items.map((w) => (w.id === id ? updated : w));
      if (this.active?.id === id) {
        this.active = updated;
      }
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    }
  }

  async remove(id: string): Promise<boolean> {
    try {
      await workspacesApi.deleteWorkspace(id);
      this.items = this.items.filter((w) => w.id !== id);
      if (this.active?.id === id) {
        this.active = this.items[0] ?? null;
      }
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    }
  }

  select(workspace: Workspace): void {
    this.active = workspace;
  }
}

export const workspaces = new WorkspacesStore();