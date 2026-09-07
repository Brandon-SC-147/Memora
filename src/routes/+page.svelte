<script lang="ts">
  import WorkspaceSidebar from "$lib/features/workspaces/workspace-sidebar.svelte";
  import ClipsView from "$lib/features/clips/clips-view.svelte";
  import DiagnosticsStatus from "$lib/features/diagnostics/diagnostics-status.svelte";
  import { Separator } from "$lib/components/ui/separator";
  import { Search, LayoutGrid } from "@lucide/svelte";
</script>

<div class="flex h-dvh w-full flex-col">
  <header
    class="flex h-12 shrink-0 items-center gap-2 border-b border-border/60 px-4 dark:border-border"
  >
    <div class="flex items-center gap-2">
      <div
        class="flex size-7 items-center justify-center rounded-lg bg-primary/10 text-primary"
      >
        <LayoutGrid class="size-4" />
      </div>
      <span class="text-sm font-semibold tracking-tight">Memora</span>
    </div>
    <div class="ml-auto flex items-center gap-2 text-xs text-muted-foreground">
      <span class="inline-flex items-center gap-1">
        <Search class="size-3" /> Buscar con <kbd class="rounded border px-1 font-mono text-[10px]">Ctrl K</kbd>
      </span>
    </div>
  </header>

  <div class="flex min-h-0 flex-1">
    <aside class="flex w-56 shrink-0 flex-col border-r border-border/60 px-3 py-3 dark:border-border">
      <WorkspaceSidebar />
    </aside>

    <div class="flex min-w-0 flex-1 flex-col bg-background">
      <div class="flex h-full flex-1 flex-col p-4">
        <ClipsView />
      </div>
    </div>
  </div>

  <footer class="flex h-9 shrink-0 items-center border-t border-border/60 px-4 dark:border-border">
    <DiagnosticsStatus />
  </footer>
</div>