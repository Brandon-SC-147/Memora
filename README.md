# Memora

**Gestor inteligente de portapapeles y conocimiento personal** — Aplicación de escritorio construida con Tauri 2, SvelteKit y Rust.

Memora captura, almacena, organiza y busca fragments de texto del portapapeles con detección automática de tipo de contenido, deduplicación por hash SHA-256 y búsqueda full-text con ranking BM25.

---

## Características

- **Captura inteligente** — Clasificación automática de contenido: texto, URLs, emails, JSON, código fuente (con detección de lenguaje), números de teléfono, colores, rutas de archivo y comandos.
- **Deduplicación** — Hash SHA-256 del contenido; los clips duplicados incrementan un contador en lugar de crear filas nuevas.
- **Espacios de trabajo** — Organización de clips en workspaces con creación, renombrado y eliminación.
- **Favoritos y fijados** — Marcado de clips importantes para acceso rápido.
- **Búsqueda full-text** — SQLite FTS5 con tokenización unicode e insensitive a diacríticos, ideal para texto en español.
- **Panel de diagnósticos** — Estado de la aplicación, salud de la base de datos y estadísticas en tiempo real.
- **Detección de secretos** — Columna `is_secret` preparada para futura detección automática (Fase 4).

---

## Arquitectura

El backend sigue una arquitectura **Domain-Driven Design (DDD)** con capas limpias:

```
src-tauri/src/
├── domain/          → Modelos, traits de repositorios, errores de dominio (sin dependencias externas)
├── application/     → Casos de uso genéricos sobre traits de dominio
└── infrastructure/  → Adaptadores: comandos Tauri, SQLite, FTS5, composición (AppState)
```

**Principios clave:**
- Cada caso de uso es genérico sobre su trait de repositorio y testeable con fakes en memoria.
- El FTS indexa dentro de la misma transacción que los inserts/clips (ADR-0002), garantizando consistencia.
- UUID v7 (time-ordered) como PK en todas las tablas.
- Schema pre-modelado para futuras fases (tags, activity log, canvas de conocimiento) evitando migraciones rotas.

---

## Tech Stack

| Capa | Tecnologías |
|---|---|
| **Frontend** | SvelteKit 2, Svelte 5 (runes), TypeScript 6, Vite 8, Tailwind CSS v4, bits-ui, Lucide icons |
| **Backend** | Tauri 2, Rust (edition 2021), sqlx (SQLite), tokio, serde, thiserror |
| **Base de datos** | SQLite con WAL, FTS5, foreign keys |
| **UI Components** | shadcn-svelte (button, card, dialog, command, tabs, tooltip, etc.) |
| **CI/CD** | GitHub Actions: check frontend, cargo test, build Tauri Windows |

---

## Estructura del proyecto

```
Memora/
├── src/                          # Frontend SvelteKit (SPA)
│   ├── lib/
│   │   ├── api/                  # Capa de transporte Tauri IPC
│   │   ├── components/ui/        # Componentes UI reutilizables
│   │   ├── features/
│   │   │   ├── clips/            # Vista y store de clips
│   │   │   ├── workspaces/       # Sidebar y store de espacios
│   │   │   ├── search/           # Store de búsqueda
│   │   │   └── diagnostics/      # Store y componente de diagnóstico
│   │   └── utils.ts              # Helper `cn()` para clases
│   └── routes/                   # Páginas SvelteKit
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── domain/               # Modelos, repositorios, errores
│   │   ├── application/          # Casos de uso
│   │   └── infrastructure/       # Tauri commands, SQLite, FTS5, estado
│   └── migrations/               # SQL migrations (embed)
├── static/                       # Assets estáticos
├── svelte.config.js
├── vite.config.js
└── package.json
```

---

## Instalación

### Requisitos

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) (stable)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Desarrollo

```bash
# Instalar dependencias
npm install

# Ejecutar en modo desarrollo
npm run tauri dev
```

### Build

```bash
# Compilar para producción
npm run tauri build
```

---

## Comandos disponibles

| Comando | Descripción |
|---|---|
| `npm run dev` | Servidor de desarrollo Vite |
| `npm run build` | Build de producción del frontend |
| `npm run check` | Verificación de tipos TypeScript/Svelte |
| `npm run tauri dev` | App completa en modo desarrollo |
| `npm run tauri build` | Empaquetar app de escritorio |

---

## Comandos IPC (Tauri)

| Comando | Descripción |
|---|---|
| `app_info` | Nombre y versión de la aplicación |
| `health` | Estado de salud: app, DB, conteo de clips y workspaces |
| `list_clips` | Listar clips con filtros (workspace, tipo, favoritos, fijados) |
| `search_clips` | Búsqueda full-text con ranking BM25 |
| `create_workspace` | Crear nuevo espacio de trabajo |
| `list_workspaces` | Listar todos los espacios |
| `rename_workspace` | Renombrar un espacio |
| `delete_workspace` | Eliminar un espacio (clips se desvinculan) |

---

## Roadmap

- [x] **Fase 1** — Clips, workspaces, búsqueda, diagnósticos
- [ ] **Fase 2** — Monitor de portapapeles en tiempo real
- [ ] **Fase 3** — Tags y etiquetado
- [ ] **Fase 4** — Detección automática de secretos
- [ ] **Fase 5** — Comportamiento de pegado
- [ ] **Fase 6** — Analytics de actividad
- [ ] **Fase 7** — Historial y deshacer
- [ ] **Fase 8** — Canvas de conocimiento (grafo nodos/aristas)

---

## Licencia

[MIT](LICENSE)
