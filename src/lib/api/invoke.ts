import { invoke } from "@tauri-apps/api/core";

/**
 * Puerta única hacia el backend Tauri.
 *
 * Se define como objeto con una función `call` para poder sustituir el
 * transporte real (Tauri IPC) por un mock en tests/screenshots sin tocar
 * el resto de la app.
 */
export interface Transport {
  call<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

export const tauriTransport: Transport = {
  call: (command, args) => invoke(command, args),
};

export const transport: Transport = tauriTransport;