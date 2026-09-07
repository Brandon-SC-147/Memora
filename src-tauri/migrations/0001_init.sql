PRAGMA foreign_keys = ON;

-- Workspaces
CREATE TABLE workspaces (
    id          TEXT PRIMARY KEY,          -- UUID v7
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    icon        TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    is_default  INTEGER NOT NULL DEFAULT 0 -- 'Favoritos'/'Inbox'
);

-- Clips
CREATE TABLE clips (
    id             TEXT PRIMARY KEY,       -- UUID v7
    content        TEXT NOT NULL,          -- texto o path/URI de imagen
    content_type   TEXT NOT NULL,          -- enum detector
    metadata       TEXT NOT NULL DEFAULT '{}',  -- JSON (language, color format, url domain...)
    content_hash   TEXT NOT NULL UNIQUE,   -- sha256 para dedupe
    is_image       INTEGER NOT NULL DEFAULT 0,
    image_path     TEXT,                   -- archivo bajo data dir
    copy_count     INTEGER NOT NULL DEFAULT 1,
    favorite       INTEGER NOT NULL DEFAULT 0,
    pinned         INTEGER NOT NULL DEFAULT 0,
    is_secret      INTEGER NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    last_copied_at TEXT NOT NULL,
    workspace_id   TEXT REFERENCES workspaces(id) ON DELETE SET NULL
);
CREATE INDEX idx_clips_created ON clips(created_at DESC);
CREATE INDEX idx_clips_workspace ON clips(workspace_id);
CREATE INDEX idx_clips_type ON clips(content_type);
CREATE INDEX idx_clips_favorite ON clips(favorite) WHERE favorite = 1;

-- Tags
CREATE TABLE tags (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- clip <-> tag
CREATE TABLE clip_tags (
    clip_id TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (clip_id, tag_id)
);
CREATE INDEX idx_clip_tags_tag ON clip_tags(tag_id);

-- Actividad (historial para analytics / undo)
CREATE TABLE activity_log (
    id          TEXT PRIMARY KEY,
    clip_id     TEXT REFERENCES clips(id) ON DELETE CASCADE,
    action      TEXT NOT NULL,   -- 'copied|open|moved|favorite|paste'
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_activity_clip ON activity_log(clip_id, occurred_at);

-- Canvas (Fase 8; modelado ahora para no romper migraciones)
CREATE TABLE canvas_nodes (
    id         TEXT PRIMARY KEY,
    clip_id    TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
    x          REAL NOT NULL,
    y          REAL NOT NULL,
    z          REAL NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_canvas_nodes_clip ON canvas_nodes(clip_id);

CREATE TABLE canvas_edges (
    id        TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES canvas_nodes(id) ON DELETE CASCADE,
    target_id TEXT NOT NULL REFERENCES canvas_nodes(id) ON DELETE CASCADE,
    relation  TEXT NOT NULL  -- 'related|source|depends|reference|belongs'
);
CREATE INDEX idx_canvas_edges_source ON canvas_edges(source_id);

-- Búsqueda (FTS5 contentful: índice separado, ver ADR-0002)
CREATE VIRTUAL TABLE clips_fts USING fts5(
    content,
    content_type,
    tokenize = 'unicode61 remove_diacritics 2'
);