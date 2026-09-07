<script lang="ts">
  import { onMount } from "svelte";

  import "../app.css";
  import { Toaster } from "$lib/components/ui/sonner";
  import { workspaces } from "$lib/features/workspaces/workspaces.store.svelte";
  import { diagnostics } from "$lib/features/diagnostics/diagnostics.store.svelte";

  onMount(() => {
    void workspaces.load();
    void diagnostics.refresh();
  });
</script>

<slot />
<Toaster />