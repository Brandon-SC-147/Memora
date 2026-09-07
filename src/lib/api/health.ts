import { transport } from "./invoke";
import type { AppInfo, HealthInfo } from "./types";

export function appInfo(): Promise<AppInfo> {
  return transport.call("app_info");
}

export function health(): Promise<HealthInfo> {
  return transport.call("health");
}