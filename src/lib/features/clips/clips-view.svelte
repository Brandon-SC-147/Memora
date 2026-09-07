<script lang="ts">
  import { onMount } from "svelte";

  import { clips } from "./clips.store.svelte";
  import { search } from "$lib/features/search/search.store.svelte";
  import ClipCard from "./clip-card.svelte";
  import { InputGroup, InputGroupAddon, InputGroupInput } from "$lib/components/ui/input-group";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Search, X, Star, Pin, ClipboardList } from "@lucide/svelte";

  let searchText = $state("");
  let debounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  onMount(() => {
    void clips.load();
  });

  function onSearchInput(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value;
    searchText = value;

    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      void search.search(value);
    }, 250);
  }

  function clearSearch() {
    searchText = "";
    search.reset();
    if (debounceTimer) clearTimeout(debounceTimer);
  }
</script>

<div class="flex h-full flex-col gap-3">
  <div class="flex shrink-0 items-center gap-2">
    <div class="relative flex-1">
      <InputGroup class="h-9 w-full">
        <InputGroupInput
          placeholder="Buscar en tu historial…"
          value={searchText}
          oninput={onSearchInput}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              if (debounceTimer) clearTimeout(debounceTimer);
              void search.search(searchText);
            }
          }}
        />
        <InputGroupAddon>
          <Search class="size-4 opacity-50" />
        </InputGroupAddon>
      </InputGroup>
      {#if searchText}
        <Button
          variant="ghost"
          size="icon-sm"
          class="absolute right-9 top-1/2 -translate-y-1/2"
          onclick={clearSearch}
          aria-label="Limpiar búsqueda"
        >
          <X class="size-4" />
        </Button>
      {/if}
    </div>

    <Button
      variant="ghost"
      size="icon"
      title="Solo favoritos"
      class={clips.filter.favoriteOnly ? "bg-primary/10" : ""}
      onclick={() => clips.setFilter({ favoriteOnly: !clips.filter.favoriteOnly })}
    >
      <Star class="size-4" />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      title="Solo fijados"
      class={clips.filter.pinnedOnly ? "bg-primary/10" : ""}
      onclick={() => clips.setFilter({ pinnedOnly: !clips.filter.pinnedOnly })}
    >
      <Pin class="size-4" />
    </Button>
  </div>

  {#if search.active}
    <!-- Resultados de búsqueda -->
    {#if search.loading}
      <div class="grid flex-1 grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {#each Array(6) as _ (Math.random())}
          <div class="h-32 animate-pulse rounded-xl bg-muted/40"></div>
        {/each}
      </div>
    {:else if search.results.length === 0}
      <div class="flex flex-1 flex-col items-center justify-center gap-2 text-center text-sm text-muted-foreground">
        <Search class="size-8 opacity-30" />
        <p>Sin resultados para «{search.query}»</p>
      </div>
    {:else}
      <div class="grid flex-1 grid-cols-1 content-start gap-3 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3">
        {#each search.results as { entry } (entry.id)}
          <ClipCard clip={entry} />
        {/each}
      </div>
    {/if}
  {:else}
    <!-- Historial -->
    <div class="flex min-h-0 flex-1 flex-col">
      <div class="mb-2 flex items-center gap-2">
        {#if clips.filter.favoriteOnly}
          <Badge variant="outline" class="text-[11px]">
            <Star class="size-3" /> Favoritos
          </Badge>
        {/if}
        {#if clips.filter.pinnedOnly}
          <Badge variant="outline" class="text-[11px]">
            <Pin class="size-3" /> Fijados
          </Badge>
        {/if}
      </div>

      {#if clips.loading && clips.items.length === 0}
        <div class="grid flex-1 grid-cols-1 content-start gap-3 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3">
          {#each Array(9) as _ (Math.random())}
            <div class="h-32 animate-pulse rounded-xl bg-muted/40"></div>
          {/each}
        </div>
      {:else if clips.items.length === 0}
        <div class="flex flex-1 flex-col items-center justify-center gap-2 text-center text-sm text-muted-foreground">
          <ClipboardList class="size-8 opacity-30" />
          <p class="font-medium text-foreground">Nada por aquí todavía</p>
          <p>Cuando copies algo, aparecerá aquí. (El monitor llega en Fase 2)</p>
        </div>
      {:else}
        <div class="grid flex-1 grid-cols-1 content-start gap-3 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3">
          {#each clips.items as clip (clip.id)}
            <ClipCard clip={clip} />
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>