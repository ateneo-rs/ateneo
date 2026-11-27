# Ateneo 0.1 — Arquitectura y orden de construcción

Nov 27, 2026 · James Paul Vizuete Davila

Este documento fija la estructura, las decisiones técnicas y el orden de trabajo de la versión 0.1 (Cimientos). Completa a `docs/planificacion.md`, que sigue siendo la referencia para el producto y los mockups.

## Alcance

| Entra en la 0.1 | Pasa a la 0.2 |
|---|---|
| Instalación por consola (cliclack), `ateneo.toml` y estados del sitio | Inicio de sesión con Google y Microsoft, y dominios permitidos |
| Modelo de datos completo; las tablas de lectores se crean, pero no se usan todavía | Cuentas de lector, "Mis datos" y vincular otro acceso |
| Cuentas del personal con correo y contraseña (argon2id), roles y 2FA (TOTP con 10 códigos de respaldo) | Paso 4 del asistente (Google o Microsoft) |
| Depósito, sitio público, búsqueda (Tantivy) y OAI-PMH | Respaldos programados y cifrados (age), con destinos OpenDAL y "Probar restauración" |
| ARK para cada obra | Handle anterior, que llega con la importación desde DSpace |
| `ateneo backup now`: local, sin cifrar, solo para datos de prueba | |
| Módulo de IA vacío (`ateneo-mod-ai`) y texto de cada página de los PDF | |

Como la 0.1 no tiene respaldos programados, el valor por defecto "Respaldos: locales, diarios a las 02:00" se activa en la 0.2.

## Workspace

```
ateneo/
├── Cargo.toml              workspace
├── deny.toml               reglas de cargo-deny
├── crates/
│   ├── ateneo/             binario
│   │   ├── src/
│   │   │   ├── cli/        init, config, admin, status, backup, reindex
│   │   │   ├── server/     arranque de Axum y rutas
│   │   │   ├── api/        API del panel
│   │   │   ├── site/       sitio público con minijinja
│   │   │   ├── oai/        OAI-PMH
│   │   │   └── panel/      archivos del panel con rust-embed
│   │   ├── templates/      plantillas minijinja
│   │   └── dist/           panel compilado (fuera de git, dentro del paquete)
│   ├── ateneo-core/
│   │   ├── migrations/     migraciones centrales
│   │   ├── .sqlx/          consultas preparadas para compilar sin base
│   │   └── src/
│   │       ├── config/     ateneo.toml y tabla settings
│   │       ├── db/         conexión, migraciones y module_migrations
│   │       ├── auth/       usuarios, sesiones, roles, permisos y 2FA
│   │       ├── ark/        generación, validación y resolución de ARK
│   │       ├── jobs/       cola de tareas y trabajadores
│   │       ├── storage/    archivos de cada obra en OCFL
│   │       ├── text/       extracción del texto de cada página
│   │       ├── items/      obras, colaboradores, revisiones y archivos
│   │       ├── search/     índice Tantivy
│   │       ├── backup/     ateneo backup now
│   │       └── module/     trait Module, registro de módulos y health
│   └── ateneo-mod-ai/      módulo de IA (vacío en la 0.1)
└── panel/                  React + MUI con Vite
```

### Crates

La regla "cada módulo en su propio crate" se aplica a los módulos que se activan desde el panel. Todo lo demás vive en dos crates.

| Crate | Contenido | Depende de |
|---|---|---|
| `ateneo` | Binario: CLI, servidor, API, sitio público, OAI-PMH y panel incluido | `ateneo-core`, `ateneo-mod-*` |
| `ateneo-core` | Todas las áreas comunes y el `trait Module` | — |
| `ateneo-mod-ai` | Módulo de IA; en la 0.1 solo se registra y declara su configuración | `ateneo-core` |

`ateneo-core` existe como crate aparte porque un módulo no puede depender del binario.

### Publicación en crates.io

`cargo install ateneo` necesita que los tres crates estén publicados.

| Nombre | Cuándo |
|---|---|
| `ateneo` | Ya publicado (0.0.1) |
| `ateneo-core` | Con la 0.1 |
| `ateneo-mod-ai` | Con la 0.1 |
| `ateneo-mod-stats`, `ateneo-mod-ocr`, `ateneo-mod-sword`, `ateneo-mod-ranking`, `ateneo-mod-library` | Cuando cada uno tenga código real |

Las reglas de crates.io no permiten reservar nombres en masa con crates vacíos, por eso los módulos futuros se publican cuando existan.

Para que el paquete compile sin PostgreSQL ni Node:

- El paquete incluye `.sqlx/` (y se compila con `SQLX_OFFLINE=true`), `migrations/`, `templates/` y el panel ya compilado en `dist/`.
- `dist/` está en `.gitignore`. Cargo omite por defecto lo que ignora git, así que `crates/ateneo/Cargo.toml` lo agrega de forma explícita en `include`.
- Cada paquete pesa menos de 10 MB (límite de crates.io); las tipografías son lo que más pesa.
- Cada `Cargo.toml` lleva `license = "AGPL-3.0-or-later"`, y las dependencias internas usan `path` y `version`.
- Se publica con `cargo publish --workspace`, que respeta el orden de dependencias.

## Modelo de datos: piezas nuevas

Las tablas de `docs/planificacion.md` se mantienen. Estas son las que la 0.1 define con detalle.

### `file_texts`

El texto de cada página de cada PDF. Lo usa el buscador desde la 0.1 y la IA después.

| Columna | Tipo | Detalle |
|---|---|---|
| `file_id` | referencia a `files` | Se borra con el archivo |
| `page` | entero | Desde 1 |
| `text` | texto | Texto de la página |
| `source` | `pdf` u `ocr` | En la 0.1 solo `pdf`; `ocr` lo llena el módulo de OCR |

Clave primaria: `(file_id, page, source)`.

### `jobs`

| Columna | Tipo | Detalle |
|---|---|---|
| `id` | identificador | |
| `kind` | texto | Por ejemplo `text.extract`, `search.index` |
| `payload` | JSONB | Datos de la tarea |
| `run_at` | fecha y hora | Desde cuándo puede ejecutarse |
| `attempts` | entero | Intentos hechos |
| `progress` | entero 0–100 | Lo que muestra la cola en el panel |
| `locked_at` | fecha y hora o nulo | Cuándo la tomó un trabajador |

También lleva `status`, `last_error` y `created_at`, necesarios para la pantalla de la cola.

- Los trabajadores toman tareas con `SELECT … WHERE run_at <= now() AND locked_at IS NULL … FOR UPDATE SKIP LOCKED`.
- Al insertar una tarea se hace `pg_notify('ateneo_jobs', …)`; los trabajadores esperan con `LISTEN ateneo_jobs`. Una revisión cada 30 segundos atiende las tareas con `run_at` futuro y recupera las que quedaron bloqueadas por un trabajador caído (`locked_at` muy antiguo).
- Una tarea que falla vuelve a la cola con espera creciente hasta su límite de intentos.

### `module_migrations`

| Columna | Detalle |
|---|---|
| `module_id` | Id del módulo |
| `version` | Número de la migración |
| `name` | Nombre |
| `checksum` | SHA-256 del SQL |
| `applied_at` | Fecha de aplicación |

## Migraciones

- **Centrales:** en `crates/ateneo-core/migrations/`, aplicadas con `sqlx::migrate!` al arrancar y en `ateneo init`.
- **De cada módulo:** `Module::migrations` devuelve una lista fija; el SQL de cada una se carga con `include_str!`. Se aplican en orden, cada una en su transacción, y se registran en `module_migrations`.
- **Solo tablas `mod_<id>_*`:** antes de confirmar la transacción, Ateneo revisa en `pg_catalog` los objetos creados; si alguno no empieza con `mod_<id>_`, deshace la migración y el módulo no se activa.
- Si el `checksum` de una migración ya aplicada cambia, el arranque se detiene con un error claro.

## Módulos y salud

- `trait Module` con `id`, `info`, `depends_on`, `settings_schema`, `migrations`, `routes`, `admin_menu`, `on_event` y `health`.
- Cada área de `ateneo-core` que lo necesita (db, storage, search, jobs, backup) expone su propia revisión de salud, y `module` reúne todas con las de los módulos activos en un único informe.
- `ateneo status` y el Inicio del panel usan ese mismo informe: base de datos, disco, índice, cola de tareas, último respaldo y módulos.

## Texto de los PDF

Al subir un PDF se crea una tarea `text.extract`. Esta tarea guarda el texto de cada página en `file_texts` con `source = pdf` y luego crea una tarea `search.index` para la obra. Los PDF escaneados quedan sin texto hasta que exista el módulo de OCR.

## Identificadores ARK

| Opción | Por defecto |
|---|---|
| `ark.naan` | `99999` (de pruebas) |
| `ark.shoulder` | `fk4` (de pruebas) |

- **NAAN:** lo normal son 5 dígitos, pero la especificación ARK actual admite otras longitudes. Por eso se acepta cualquier cadena no vacía de caracteres betanuméricos (`0123456789bcdfghjkmnpqrstvwxz`) y se rechaza todo lo demás.
- **Shoulder:** caracteres betanuméricos que terminan en el primer dígito (convención "first-digit"), por ejemplo `fk4`, `x5` o `b7`.
- **Forma:** `ark:/<naan>/<shoulder><8 caracteres al azar><carácter de control>`. El carácter de control usa el algoritmo NCDA. El identificador es opaco y no revela el número interno de la obra.
- **Asignación:** al crear la obra, una sola vez. Se guarda completo en `items.ark` con índice único, así que cambiar el NAAN solo afecta a las obras nuevas. Una obra retirada conserva su ARK y muestra una página que lo indica.
- **Rutas:** `/ark:/<naan>/<nombre>` y `/ark:<naan>/<nombre>` redirigen a `/obra/{id}/{slug}`; con `?info` devuelven una ficha breve de metadatos.
- **Dónde aparece:** ficha de la obra, etiquetas `citation_*`, schema.org y `dc.identifier` en OAI-PMH.
- **Configuración:** `ateneo config set ark.naan <valor>`, `ateneo config set ark.shoulder <valor>` y el paso 5 del asistente.
- **Aviso:** mientras el NAAN sea `99999`, el Inicio del panel muestra "Pendientes: NAAN de prueba". Un NAAN propio se pide gratis en arks.org.

## `ateneo backup now`

Al ejecutarse muestra este aviso y pide confirmar (o acepta `--yes`):

> ⚠ Respaldo local y SIN CIFRAR. Úsalo solo con datos de prueba. Los respaldos cifrados llegan en la 0.2.

Crea `respaldos/ateneo-AAAAMMDD-HHMMSS.tar.zst` con:

- `db.dump`: `pg_dump -Fc` de la base;
- `objetos/`: los archivos OCFL (el índice no, porque se regenera);
- `manifest.json`: versiones de Ateneo y de PostgreSQL, y el SHA-256 de cada parte.

Se registra en `backups` con `encrypted = false` y en la auditoría.

### `pg_dump` de la misma versión del servidor

1. Ateneo lee `SHOW server_version_num` (por ejemplo, `170004` corresponde a la versión mayor 17).
2. Busca `pg_dump` en este orden:
   - `backup.pg_dump_path`, si está configurado;
   - las rutas de esa versión: `/usr/lib/postgresql/<N>/bin` (Debian y Ubuntu), `/usr/pgsql-<N>/bin` (RHEL), `postgresql@<N>` de Homebrew y `C:\Program Files\PostgreSQL\<N>\bin`;
   - el `PATH`.
3. Ejecuta `pg_dump --version` y solo acepta la misma versión mayor. Un `pg_dump` más nuevo puede leer un servidor viejo, pero su salida no siempre se restaura en ese servidor; por ejemplo, el 17 escribe `SET transaction_timeout`, que el 16 no entiende. Si no lo encuentra, se detiene y dice qué paquete instalar.
4. La contraseña va en la variable `PGPASSWORD` del proceso hijo, nunca en la línea de comandos.

**Con Docker:** el contenedor de Ateneo no ejecuta comandos en el contenedor `db`, porque eso exigiría montar el socket de Docker. La imagen de Ateneo instala desde el repositorio PGDG los clientes de todas las versiones de PostgreSQL con soporte (`postgresql-client-15` a `-18`, y la 19 si ya salió al publicar). Cada uno queda en `/usr/lib/postgresql/<N>/bin`, donde el paso 2 lo encuentra. El `docker compose` fija `postgres:17`, y si alguien cambia la versión sigue funcionando.

En la 0.1 no hay comando para restaurar; la documentación explica cómo hacerlo con `pg_restore`.

## Panel

- Vive en `panel/` (React, MUI y Vite, con pnpm).
- Vite lo compila a `crates/ateneo/dist/`, que rust-embed incluye en el binario. En modo debug, rust-embed lee los archivos del disco, así que basta con recompilar el panel.
- `dist/` queda fuera de git pero dentro del paquete de crates.io.
- Solo se usan `@mui/material` y la edición comunitaria de MUI X (`@mui/x-data-grid`, `@mui/x-date-pickers`), con licencia MIT. Nunca las ediciones Pro ni Premium; el CI falla si aparece algún paquete `@mui/x-*-pro`, `@mui/x-*-premium` o `@mui/x-license`.

## Dependencias principales

Versiones vigentes al 27 de noviembre de 2026.

### Rust

| Crate | Versión | Licencia | Uso |
|---|---|---|---|
| tokio | 1.54 | MIT | Ejecución asíncrona |
| axum | 0.8 | MIT | Servidor web |
| tower-http | 0.7 | MIT | Compresión, archivos estáticos, cabeceras |
| sqlx | 0.9 | MIT OR Apache-2.0 | PostgreSQL y migraciones |
| tantivy | 0.27 | MIT | Buscador |
| minijinja | 3.1 | Apache-2.0 | Plantillas del sitio público |
| rust-embed | 8.14 | MIT | Panel dentro del binario |
| cliclack | 0.5 | MIT | Preguntas en la consola |
| clap | 4.6 | MIT OR Apache-2.0 | Comandos de la consola |
| serde / serde_json | 1.0 | MIT OR Apache-2.0 | Serialización |
| toml | 1.2 | MIT OR Apache-2.0 | `ateneo.toml` |
| argon2 | 0.6 | MIT OR Apache-2.0 | Contraseñas |
| totp-rs | 6.0 | MIT | 2FA |
| rand | 0.10 | MIT OR Apache-2.0 | ARK, códigos de respaldo y tokens |
| pdf-extract | 0.12 | MIT | Texto de cada página |
| lopdf | 0.47 | MIT | Lectura de PDF |
| quick-xml | 0.42 | MIT | OAI-PMH |
| zstd | 0.14 | BSD-3-Clause | Compresión del respaldo |
| tar | 0.4 | MIT OR Apache-2.0 | Archivo del respaldo |
| sha2 | 0.11 | MIT OR Apache-2.0 | Sumas de verificación (OCFL, respaldos, migraciones) |
| time | 0.3 | MIT OR Apache-2.0 | Fechas |
| uuid | 1.29 | Apache-2.0 OR MIT | Identificadores |
| tracing / tracing-subscriber | 0.1 / 0.3 | MIT | Registros |
| thiserror / anyhow | 2.0 / 1.0 | MIT OR Apache-2.0 | Errores |

### Panel

| Paquete | Versión | Licencia |
|---|---|---|
| react / react-dom | 19.3 | MIT |
| react-router | 8.5 | MIT |
| @mui/material | 9.6 | MIT |
| @mui/x-data-grid | 9.18 | MIT |
| @mui/x-date-pickers | 9.18 | MIT |
| @emotion/react / @emotion/styled | 11.14 | MIT |
| vite | 8.4 | MIT |
| @vitejs/plugin-react | 6.3 | MIT |
| typescript | 7.0 | Apache-2.0 |

## CI (GitHub Actions)

| Trabajo | Qué revisa |
|---|---|
| fmt y clippy | `cargo fmt --check` y `cargo clippy -- -D warnings` |
| test | Pruebas contra PostgreSQL 15 a 18 (matriz), incluida la búsqueda de `pg_dump` |
| deny | `cargo-deny` con `EmbarkStudios/cargo-deny-action` |
| reglas del proyecto | Encabezado SPDX en cada archivo, firma DCO y Conventional Commits |
| panel | `pnpm install --frozen-lockfile`, `pnpm build` y que no haya paquetes MUI X Pro o Premium |
| paquete | `cargo publish --workspace --dry-run`, para detectar si falta `.sqlx/` o `dist/` |

### `deny.toml`

```toml
[licenses]
allow = [
  "AGPL-3.0-or-later", "GPL-3.0-or-later", "LGPL-2.1-or-later", "LGPL-3.0-or-later",
  "MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause", "BSD-3-Clause",
  "ISC", "Zlib", "0BSD", "BSL-1.0", "CC0-1.0", "Unlicense", "MPL-2.0",
  "Unicode-3.0", "CDLA-Permissive-2.0",
]
confidence-threshold = 0.93

[advisories]
yanked = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

Solo se permiten licencias compatibles con AGPL-3.0-or-later. Quedan fuera GPL-2.0-only, SSPL, BUSL y los crates sin licencia. Una licencia nueva (por ejemplo, OpenSSL) se agrega solo después de revisarla.

## Orden de construcción

Cada paso deja algo que funciona y se puede probar antes del siguiente.

| # | Paso | Resultado |
|---|---|---|
| 1 | `config` y `db` | Lectura de `ateneo.toml`, conexión y migraciones centrales |
| 2 | Binario con `init`, `config`, `config set` y `status` | Instalación por consola y primer informe de salud |
| 3 | `auth`, con `admin create` y `admin reset` | Super admin, sesiones, roles y 2FA |
| 4 | `module` y `jobs` | `trait Module`, `module_migrations`, cola con trabajadores |
| 5 | `storage` y `text` | Archivos en OCFL y texto de cada página en `file_texts` |
| 6 | `items` y `ark` | Obras con su ARK y su resolución |
| 7 | API y panel | Panel incluido en el binario, depósito desde el panel |
| 8 | `search` y `reindex` | Índice Tantivy y `ateneo reindex` |
| 9 | Sitio público | Portada, colecciones, ficha de obra, búsqueda |
| 10 | OAI-PMH | Cosecha con `dc.identifier` = ARK |
| 11 | `backup now` | Respaldo local sin cifrar con aviso |
| 12 | `ateneo-mod-ai` | Módulo vacío registrado y visible en Módulos |
