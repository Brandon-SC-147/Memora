<script lang="ts">
  import { onMount } from "svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { diagnostics } from "./diagnostics.store.svelte";

  onMount(() => {
    void diagnostics.refresh();
  });
</script>

{#if diagnostics.info}
  <div
    class="flex items-center gap-2 rounded-lg border-border/60 bg-muted/30 px-3 py-1.5 text-xs text-muted-foreground"
  >
    <span class="font-medium">{diagnostics.info.app.name} v{diagnostics.info.app.version}</span>
    <Badge
      variant={diagnostics.info.databaseHealthy ? "default" : "destructive"}
      class="h-4 gap-1 px-1.5 text-[10px] leading-none"
    >
      <span
        class="size-1.5 rounded-full {diagnostics.info.databaseHealthy ? 'bg-green-500' : 'bg-red-500'}"
      ></span>
      {#if diagnostics.info.databaseHealthy}
        DB
      {:else}
        DB error
      {/if}
    </Badge>
    <span>{diagnostics.info.clipCount} clips</span>
    <span>{diagnostics.info.workspaceCount} espacios</span>
  </div>
{:else if diagnostics.loading}
  <div class="h-7 w-56 animate-pulse rounded-lg bg-muted/40"></div>
{:else if diagnostics.error}
  <div class="text-xs text-destructive">No se pudo contactar al backend</div>
{/if}