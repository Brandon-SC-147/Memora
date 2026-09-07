<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  import { workspaces } from "./workspaces.store.svelte";
  import { clips } from "$lib/features/clips/clips.store.svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from "$lib/components/ui/dropdown-menu";
  import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Plus, Pencil, Trash2, MoreHorizontal } from "@lucide/svelte";

  let dialogOpen = $state(false);
  let editing: { id: string; name: string } | null = $state(null);
  let newName = $state("");

  onMount(() => {
    void workspaces.load();
  });

  function openCreate() {
    editing = null;
    newName = "";
    dialogOpen = true;
  }

  function openRename(ws: { id: string; name: string }) {
    editing = { id: ws.id, name: ws.name };
    newName = ws.name;
    dialogOpen = true;
  }

  async function submit() {
    const name = newName.trim();
    if (!name) {
      toast.error("El nombre no puede estar vacío");
      return;
    }
    const ok = editing
      ? await workspaces.rename(editing.id, name)
      : await workspaces.create(name);
    if (ok) {
      toast.success(editing ? "Espacio renombrado" : "Espacio creado");
    } else if (workspaces.error) {
      toast.error(workspaces.error);
    }
    dialogOpen = false;
  }

  async function remove(id: string, name: string) {
    const ok = await workspaces.remove(id);
    if (ok) {
      toast.success(`Espacio "${name}" eliminado`);
      void clips.load();
    } else if (workspaces.error) {
      toast.error(workspaces.error);
    }
  }

  function select(ws: (typeof workspaces.items)[number]) {
    workspaces.select(ws);
    clips.setFilter({ workspaceId: ws.id });
  }
</script>

<div class="flex flex-col gap-1">
  <div class="flex items-center justify-between px-1 pb-1">
    <span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
      Espacios
    </span>
    <Button variant="ghost" size="icon-xs" onclick={openCreate} aria-label="Nuevo espacio">
      <Plus class="size-3.5" />
    </Button>
  </div>

  {#if workspaces.loading && workspaces.items.length === 0}
    <div class="space-y-1">
      <div class="h-8 animate-pulse rounded-lg bg-muted/40"></div>
      <div class="h-8 animate-pulse rounded-lg bg-muted/40"></div>
    </div>
  {:else if workspaces.items.length === 0}
    <p class="px-1 py-2 text-xs text-muted-foreground">
      Sin espacios todavía. Crea el primero.
    </p>
  {:else}
    {#each workspaces.items as ws (ws.id)}
      <div
        class="group flex items-center rounded-lg px-1 text-sm transition-colors"
        class:cursor-pointer={!ws.isDefault}
        role="button"
        tabindex="0"
        onclick={() => select(ws)}
        onkeydown={(e) => e.key === "Enter" && select(ws)}
      >
        <span
          class="flex h-7 flex-1 items-center gap-2 rounded-r-lg pr-1 {workspaces.active?.id === ws.id ? 'bg-primary/10' : ''}"
        >
          <input
            type="radio"
            name="workspace"
            checked={workspaces.active?.id === ws.id}
            readonly
            class="size-3.5 accent-primary"
          />
          <span
            class="truncate font-medium {workspaces.active?.id === ws.id ? 'text-primary' : ''}"
          >
            {ws.name}
          </span>
        </span>
        <DropdownMenu>
          <DropdownMenuTrigger
            class="rounded-md p-1.5 text-muted-foreground opacity-0 hover:text-foreground transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
          >
            <MoreHorizontal class="size-4" />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onclick={() => openRename(ws)}>
              <Pencil class="size-4" />
              Renombrar
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              variant="destructive"
              onclick={() => remove(ws.id, ws.name)}
              disabled={ws.isDefault}
            >
              <Trash2 class="size-4" />
              Eliminar
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    {/each}
  {/if}
</div>

<Dialog bind:open={dialogOpen}>
  <DialogContent class="sm:max-w-sm">
    <DialogHeader>
      <DialogTitle>{editing ? "Renombrar espacio" : "Nuevo espacio"}</DialogTitle>
      <DialogDescription>
        {editing
          ? "Cambia el nombre de este espacio de trabajo."
          : "Agrupa clips por proyecto o tema."}
      </DialogDescription>
    </DialogHeader>
    <form class="space-y-3" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <div class="space-y-1.5">
        <Label for="ws-name">Nombre</Label>
        <Input
          id="ws-name"
          bind:value={newName}
          placeholder="Ej. Frontend"
          autofocus
          onkeydown={(e) => e.key === "Enter" && submit()}
        />
      </div>
      <DialogFooter>
        <DialogClose>
          <Button type="button" variant="outline">Cancelar</Button>
        </DialogClose>
        <Button type="submit" onclick={submit}>Guardar</Button>
      </DialogFooter>
    </form>
  </DialogContent>
</Dialog>