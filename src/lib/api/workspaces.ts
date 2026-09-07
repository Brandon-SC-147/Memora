import { transport } from "./invoke";
import type { Workspace } from "./types";

export function listWorkspaces(): Promise<Workspace[]> {
  return transport.call("list_workspaces");
}

export interface CreateWorkspaceParams {
  name: string;
  description?: string | null;
}

export function createWorkspace(params: CreateWorkspaceParams): Promise<Workspace> {
  return transport.call("create_workspace", { params });
}

export function renameWorkspace(id: string, name: string): Promise<Workspace> {
  return transport.call("rename_workspace", { params: { id, name } });
}

export function deleteWorkspace(id: string): Promise<void> {
  return transport.call("delete_workspace", { params: { id } });
}