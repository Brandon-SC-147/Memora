<script lang="ts">
  import type { ClipboardEntry } from "$lib/api";
  import { clips as clipsStore } from "./clips.store.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from "$lib/components/ui/dropdown-menu";
  import { Star, Pin, Copy, MoreHorizontal } from "@lucide/svelte";
  import { toast } from "svelte-sonner";

  let { clip }: { clip: ClipboardEntry } = $props();

  function contentTypeLabel(raw: string): string {
    if (raw.startsWith("code:")) return raw.slice(5);
    const map: Record<string, string> = {
      text: "texto",
      url: "enlace",
      email: "email",
      json: "json",
      phone: "tel",
      color: "color",
      file_path: "ruta",
      command: "comando",
      other: "otro",
    };
    return map[raw] ?? raw;
  }

  function relative(ts: string): string {
    const time = new Date(ts).getTime();
    if (Number.isNaN(time)) return ts;
    const diff = Date.now() - time;
    if (diff < 60_000) return "ahora";
    if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} min`;
    if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} h`;
    return `${Math.floor(diff / 86_400_000)} d`;
  }

  function toggleFavorite() {
    clipsStore.toggleFavorite(clip.id);
    toast.success(clip.favorite ? "Quitado de favoritos" : "Marcado como favorito");
  }

  function togglePinned() {
    clipsStore.togglePinned(clip.id);
    toast.success(clip.pinned ? "Desfijado" : "Fijado al inicio");
  }

  function copy() {
    toast.success("Copiado al portapapeles (comportamiento completo en Fase 2)");
  }
</script>

<div
  class="group relative flex flex-col gap-2 rounded-xl border border-border/60 bg-card p-3 shadow-sm transition-shadow hover:shadow-md dark:bg-card/50"
>
  <div class="flex items-start justify-between gap-2">
    <div class="flex flex-wrap items-center gap-1.5">
      <Badge variant="secondary" class="text-[10px] uppercase">{contentTypeLabel(clip.contentType)}</Badge>
      {#if clip.favorite}
        <Star class="size-3.5 fill-amber-400 text-amber-400" />
      {/if}
      {#if clip.pinned}
        <Pin class="size-3.5 fill-sky-400 text-sky-400" />
      {/if}
    </div>
    <DropdownMenu>
      <DropdownMenuTrigger
        class="rounded-md p-1 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
        aria-label="Acciones del clip"
      >
        <MoreHorizontal class="size-4" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onclick={copy}>
          <Copy class="size-4" />
          Copiar
        </DropdownMenuItem>
        <DropdownMenuItem onclick={toggleFavorite}>
          <Star class="size-4" />
          {clip.favorite ? "Quitar favorito" : "Favorito"}
        </DropdownMenuItem>
        <DropdownMenuItem onclick={togglePinned}>
          <Pin class="size-4" />
          {clip.pinned ? "Desfijar" : "Fijar"}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  </div>

  <p class="line-clamp-3 whitespace-pre-wrap break-words text-sm text-foreground">
    {clip.content}
  </p>

  <div class="mt-auto flex items-center justify-between pt-1 text-[11px] text-muted-foreground">
    <span class="flex shrink-0 items-center gap-2">
      <span title={`Copias: ${clip.copyCount}`}>
        <Copy class="mr-0.5 inline size-3" />{clip.copyCount}
      </span>
      <span>{relative(clip.lastCopiedAt)}</span>
    </span>
  </div>
</div>