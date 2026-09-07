import { healthApi } from "$lib/api";
import type { HealthInfo } from "$lib/api";

/**
 * Estado del panel de diagnóstico (feature diagnostics).
 * Se muestra en la barra de estado como indicador de salud de la DB.
 */
export class DiagnosticsStore {
  info = $state<HealthInfo | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async refresh(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.info = await healthApi.health();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }
}

export const diagnostics = new DiagnosticsStore();