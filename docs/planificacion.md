# Ateneo — Investigación y planificación

Nov 27, 2026 · James Paul Vizuete Davila

Transcripción en Markdown de los tres documentos de `docs/planificacion/`:

1. [Idea y arquitectura](#parte-1--idea-y-arquitectura) (`01-idea-y-arquitectura.pdf`)
2. [Sitio público](#parte-2--sitio-público) (`02-sitio-publico.pdf`)
3. [Panel](#parte-3--panel) (`03-panel.pdf`)

Los mockups solo se describen aquí; para verlos, abre los PDF.

---

# Parte 1 — Idea y arquitectura

La fase de idea de Ateneo está terminada: este documento reúne todo lo decidido para construir un repositorio institucional en Rust, más simple y atractivo que DSpace, con un mockup de cada pantalla.

## Qué es Ateneo

**Ateneo** significa **Archivo de Tesis, Ensayos, Notas, Ediciones y Obras**. Es un repositorio institucional de acceso abierto para guardar, publicar y preservar la producción académica de una institución. Un ateneo es, además, un lugar de estudio y debate.

| Dato | Valor |
|---|---|
| Autor y titular | James Paul Vizuete Davila |
| Licencia del código | AGPL-3.0-or-later |
| Origen | Hecho en Ecuador, la Mitad del Mundo |
| Institución de pruebas | ISTS - Por confirmar |
| Repositorio | [github.com/ateneo-rs/ateneo](https://github.com/ateneo-rs/ateneo) |
| Contacto | jamespnpm1624@outlook.com |

### Nombres reservados

| Plataforma | Nombre | Estado |
|---|---|---|
| npm | `ateneo` | Publicado, versión 0.0.1 |
| npm (organización) | `@ateneo-rs` | Reservada para los paquetes por plataforma |
| crates.io | `ateneo` | Publicado, versión 0.0.1 |
| GitHub | Organización `ateneo-rs` | Creada, con el repositorio `ateneo` |

El sufijo `-rs` indica Rust; se usó porque `ateneo` ya estaba ocupado como organización en GitHub y en npm.

### Nombres anteriores

| Nombre | Significado | Por qué se dejó |
|---|---|---|
| Lato | Escrito con la letra "o" (no con cero). Alude a la latitud 0, la Mitad del Mundo: identidad ecuatoriana con proyección internacional | Ya tenía propietario: estaba inscrito como proyecto privado en crates.io y en npm |
| Equo | Alude a la línea equinoccial, de donde viene el nombre de Ecuador | Se dejó para buscar un nombre a la altura del calibre del proyecto, que fue Ateneo. Además, npm rechazó `equo` por parecerse a `expo` y la organización `equo` en GitHub estaba ocupada. Hoy `@equo/cli` está marcado como obsoleto y el crate `equo` fue retirado |

### Principios

- **Todo desde la interfaz.** En la consola solo se instala y se responden unas preguntas básicas.
- **Cada institución con su identidad:** nombre, logo, colores y tipografía propios.
- **Lo publicado es libre:** leer, descargar y citar nunca exige cuenta.
- **Privacidad por diseño:** sin rastreadores de terceros y sin bloquear funciones.
- **Gratis para siempre:** sin versión pagada ni funciones bloqueadas.

## Por qué una alternativa a DSpace

DSpace funciona, pero instalarlo y personalizarlo exige mucho trabajo técnico; Ateneo busca competir con él con un solo programa, menos recursos y todo configurable desde la interfaz.

| Aspecto | DSpace 7/8 | Ateneo |
|---|---|---|
| Piezas a instalar | Backend Java, Tomcat, Solr, PostgreSQL y frontend Angular | Un solo binario y PostgreSQL |
| Configuración | Gran parte en archivos y consola | Asistente y panel web |
| Identidad visual | Requiere programar un tema | Logo, colores y tipografía desde el panel |
| Lectura | Visor básico | Visor con citar, resaltar, notas y fichas |
| Privacidad | Ventana de cookies en todas las visitas | Sin ventana, sin terceros |

### Otras alternativas revisadas

| Sistema | Qué es | Por qué no resuelve lo que busca Ateneo |
|---|---|---|
| EPrints | Repositorio clásico en Perl | Tecnología antigua y poca presencia en Latinoamérica |
| InvenioRDM | Plataforma de CERN, base de Zenodo | Potente pero pesada: varios servicios aparte |
| Islandora | Repositorio sobre Drupal y Fedora | Muchas piezas que mantener |
| Samvera / Hyrax | Repositorio en Ruby on Rails | Pensado para equipos técnicos grandes |
| Dataverse | Repositorio de datos de investigación | Enfocado en datos, no en tesis y obras |

Ninguna ofrece a la vez instalación sencilla, identidad propia sin programar y herramientas de lectura modernas.

## Arquitectura

Ateneo es un solo programa en Rust que contiene el servidor, el buscador, el sitio público y el panel; solo necesita PostgreSQL al lado.

> **Diagrama · arquitectura: un binario y sus servicios externos** ("Un solo binario contiene todo y solo PostgreSQL va al lado").
> A la izquierda, quienes lo usan: Lectores (navegador), Personal (panel) y Cosechadores (OAI-PMH y Google). En el centro, el binario de Ateneo con Sitio público (minijinja), Panel (React + MUI), Servidor web y API (Axum sobre tokio), Buscador (Tantivy), Cola de tareas (OCR e importaciones) y Módulos (se activan desde el panel).
> A la derecha, los servicios externos: PostgreSQL (datos y cola), Archivos (OCFL, disco o S3), Respaldos (cifrados, regla 3-2-1) e Inicio de sesión (Google o Microsoft).

| Pieza | Tecnología |
|---|---|
| Lenguaje | Rust, edición 2024 |
| Servidor web | Axum sobre tokio |
| Base de datos | PostgreSQL (único motor soportado), con sqlx |
| Buscador | Tantivy, integrado (sin Solr) |
| Sitio público | Plantillas minijinja generadas en el servidor, legibles por Google |
| Panel | React + MUI |
| Archivos | OCFL en disco o en almacenamiento compatible con S3 |
| Tareas en segundo plano | Cola en PostgreSQL con SKIP LOCKED |
| Preguntas de instalación | cliclack, al estilo de Vite |

### Módulos

Todos los módulos vienen dentro del programa y se activan o desactivan desde el panel. Cada uno vive en su propio crate e implementa un `trait Module` con `id`, `info`, `depends_on`, `settings_schema`, `migrations` (tablas con prefijo `mod_*`), `routes`, `admin_menu`, `on_event` y `health`. Las extensiones de la comunidad en WebAssembly quedan para después de la 1.0.

### Distribución

| Canal | Cómo |
|---|---|
| npm / pnpm | El paquete `ateneo` instala el binario correcto mediante `@ateneo-rs/<plataforma>` (patrón de esbuild) |
| Docker | Imagen oficial con un `docker compose` que incluye PostgreSQL |
| Binario | Descarga desde GitHub, versiones firmadas |
| crates.io | `cargo install ateneo` |

## Modelo de datos

El modelo es híbrido: lo que se filtra y ordena va en columnas normales, y los metadatos completos van en una columna JSONB con la forma de la API REST de DSpace 7+. Se descartó el modelo EAV (una fila por metadato).

- **Columnas de `items`:** título, año, idioma, estado, acceso e identificadores.
- **`metadata` (JSONB):** `{"dc.title":[{"value":"…","language":"es","person_id":null}]}`, con índice GIN `jsonb_path_ops`.
- **Texto de cada PDF:** se guarda aparte desde la 0.1; lo usan el buscador y, más adelante, la IA.

| Grupo | Tablas |
|---|---|
| Obras | `items`, `item_contributors`, `item_revisions` (foto de cada versión), `import_records`, `files` |
| Organización | `containers` (árbol de comunidades y colecciones), `metadata_fields` (etiqueta y mapeos) |
| Cuentas | `users`, `user_emails`, `user_identities` (varias formas de entrar por persona), `sessions`, `roles`, `role_assignments` |
| Lectores | Bibliografía, notas y resaltados de cada cuenta |
| Migración | `legacy_urls` |
| Seguridad | `audit_log`, `blocked_ips` |
| Estadísticas | `usage_events`, `usage_daily`, `search_log` |
| Sistema | `settings`, `modules`, `jobs`, `notifications`, `backups`, páginas legales con versiones |
| Módulos | Tablas propias con prefijo `mod_*` |

## Instalación y primer arranque

En la consola solo se instala y se responden unas preguntas básicas; todo lo demás se configura en la interfaz.

### Métodos de instalación

| Método | Comandos | PostgreSQL |
|---|---|---|
| Docker | `docker compose up -d` y `docker compose exec ateneo ateneo init` | Incluido |
| pnpm | `pnpm add -g ateneo` y `ateneo` | Aparte |
| npm | `npm install -g ateneo` y `ateneo` | Aparte |
| Binario | Descargar de GitHub y ejecutar `ateneo` | Aparte |
| Automático | Variables de entorno; la contraseña se lee de un archivo, nunca del comando | Según el caso |

### Preguntas en la consola

Al ejecutarse por primera vez, Ateneo pregunta el idioma del instalador, el dominio, la base de datos (con Docker se detecta sola), el nombre, correo y contraseña del super admin (mínimo 15 caracteres), el idioma del sitio y la zona horaria. Como la cuenta se crea en la consola, nadie puede adelantarse a crearla desde el navegador.

> **Mockup · preguntas de instalación en la consola** ("La consola solo pregunta lo básico, paso a paso").
> Una terminal que ejecuta `docker compose exec ateneo ateneo init` y muestra las preguntas al estilo cliclack: idioma del instalador (Español), dominio (repositorio.demo.edu.ec), base de datos (PostgreSQL detectado por Docker), nombre y correo del super administrador, contraseña de mínimo 15 caracteres con confirmación, idioma principal y zona horaria (America/Guayaquil, detectada) y "¿Todo correcto?".
> Termina con "Listo. Abre https://repositorio.demo.edu.ec e inicia sesión".

### Estados del sitio

| Estado | Cuándo | Qué ve el público |
|---|---|---|
| Instalación incompleta | Antes de terminar la consola | Un aviso para quien administra el servidor |
| Recién instalado | Hasta que el super admin entra por primera vez | La portada de Ateneo por defecto, como hace DSpace |
| Publicado | Al terminar el asistente | El repositorio con la identidad de la institución |
| En construcción | Solo si se eligieron los valores por defecto | Un aviso hasta presionar "Publicar sitio"; el personal ve una vista previa |

> **Mockup · instalación incompleta y en construcción** ("Dos pantallas avisan que el sitio aún no está listo").
> A la izquierda, antes de terminar la consola: un ícono de alerta con "Instalación incompleta. Termínala en la consola del servidor".
> A la derecha, solo si se eligieron los valores por defecto: el nombre "Instituto Tecnológico Demo", el texto "Este repositorio estará disponible pronto" y un botón "Vista previa para el personal".

> **Mockup · portada de Ateneo por defecto** ("Recién instalado, el sitio muestra la portada de Ateneo").
> Una cabecera con el logo de Ateneo y los enlaces Buscar, Colecciones e Iniciar sesión, el título "Bienvenido a tu nuevo repositorio" y un aviso para iniciar sesión como super administrador, con un buscador de tesis, artículos y autores.
> Debajo hay tres tarjetas (Qué es Ateneo, Documentación y Obras de ejemplo) y un pie con "Hecho con Ateneo" y "Cookies · Cómo usamos tus datos".

### Asistente de configuración

Al entrar por primera vez, el super admin elige **Configurar paso a paso** o **Usar valores por defecto**. Cada paso tiene "Omitir" y el asistente se puede reabrir en **Sistema → Asistente de configuración**.

1. Institución: nombre, siglas, logo, ícono y sitio web
2. Apariencia: colores y tipografía
3. Idioma y región de las citas
4. Inicio de sesión con Google o Microsoft y dominios permitidos
5. Identificadores: handle anterior y ARK
6. Colecciones iniciales: por carreras, por tipo de obra o vacío
7. Datos legales: razón social, dirección, contacto y responsable de protección de datos
8. Importar desde DSpace (opcional)
9. Respaldos: destino y frecuencia
10. Resumen y publicar

> **Mockup · bienvenida y paso Institución** ("El super admin elige: asistente paso a paso o valores por defecto").
> A la izquierda, "Bienvenido a Ateneo. ¿Configurar tu repositorio ahora? Toma unos 10 minutos." con los botones "Configurar paso a paso" y "Usar valores por defecto", y la nota "Puedes cambiar todo después en el panel".
> A la derecha, el "Paso 1 de 10 · Institución" con 10 puntos de avance, los campos nombre de la institución y siglas, una zona para arrastrar el logo y los botones Omitir y Siguiente.

> **Mockup · paso Datos legales y resumen final** ("El asistente pide los datos legales y termina con un resumen").
> A la izquierda, el "Paso 7 de 10 · Datos legales" con razón social, dirección, correo de contacto y responsable de protección de datos, y la nota "Con estos datos se generan las páginas legales".
> A la derecha, el "Paso 10 de 10 · Resumen" (institución, tipografía Literata + Atkinson, inicio de sesión Google · demo.edu.ec, colecciones por carreras y respaldos diarios a las 02:00) con el botón "Terminar y publicar".

### Valores por defecto

| Opción | Valor |
|---|---|
| Nombre | "Repositorio institucional" |
| Logo, ícono, colores y tipografía | Los de Ateneo |
| Idioma | Español |
| Inicio de sesión | Solo el super admin |
| Colecciones | Una sola, "General" |
| Respaldos | Locales, diarios a las 02:00 |
| Módulos | Estadísticas activadas; ranking, biblioteca e IA desactivados |

Todo se cambia después en **Sistema → Institución**, solo por el super admin o quien él autorice; el Inicio del panel lista los "Pendientes de configuración".

### Comandos de mantenimiento

Lo que conviene no cambiar desde la web, porque un error dejaría a todos sin acceso, se cambia en la consola con un menú del mismo estilo.

| Comando | Para qué |
|---|---|
| `ateneo config` | Cambiar dominio, puerto, base de datos, idioma, zona horaria y datos del super admin |
| `ateneo config set <opción> <valor>` | Lo mismo sin menú, para scripts |
| `ateneo admin reset` | Restablecer la contraseña del super admin y desactivar su 2FA |
| `ateneo admin create` | Crear otro super admin de emergencia |
| `ateneo status` | Ver la salud del sistema |
| `ateneo backup now` | Hacer un respaldo inmediato |
| `ateneo reindex` | Reconstruir el índice del buscador |

Con Docker se antepone `docker compose exec ateneo`. Estos comandos solo funcionan con acceso a la consola del servidor, muestran el valor anterior y el nuevo antes de confirmar, quedan en la auditoría y avisan a todos los super admins. Al cambiar el dominio, recuerdan actualizar el DNS, el certificado y la dirección de retorno de Google o Microsoft; el dominio anterior redirige al nuevo y los ARK y handles no cambian. En el panel, el dominio y la base de datos se ven solo como lectura.

Fuera de Ateneo quedan configurar el DNS, instalar PostgreSQL y renovar el certificado si no es automático; se explican en **Sistema → Instrucciones manuales**.

### Carpetas en el servidor

```
/opt/ateneo/
├── ateneo            el programa
├── ateneo.toml       dominio y conexión a la base
├── datos/
│   ├── objetos/      archivos de cada obra (OCFL)
│   ├── indice/       buscador, regenerable
│   ├── fuentes/      tipografías instaladas
│   └── temporal/     subidas en proceso y OCR
├── respaldos/        copias locales
└── registros/        logs
```

## Marca y protección

El código es abierto y cualquiera puede copiarlo, pero nadie puede hacer pasar una copia por Ateneo: así protegen su nombre Firefox, Mattermost y Plausible.

| Capa | Qué protege | Estado |
|---|---|---|
| Derecho de autor | El código; existe sin trámite (Decisión 351, art. 52) | Vigente |
| Licencia AGPL-3.0 | Toda copia mantiene el nombre del autor y la licencia, y publica sus cambios aunque solo se ofrezca como servicio web | Vigente |
| `TRADEMARK.md` | El nombre y el logo: una versión modificada debe llevar otro nombre (lo permite la sección 7 de la AGPLv3) | Publicado |
| Registro en el SENADI | Uso exclusivo del nombre en Ecuador | Fase prototipo |
| Canales oficiales | Sitio, organización `ateneo-rs`, paquetes `ateneo` y versiones firmadas | Vigente |

### Política de marca

- **Permitido sin pedir permiso:** usar Ateneo sin cambios, decir "hecho con Ateneo" o "compatible con Ateneo", escribir sobre él y decir que un fork está "basado en Ateneo".
- **Requiere otro nombre:** distribuir una versión modificada u ofrecer un servicio en línea con ella.
- **No permitido:** aparentar ser oficial, usar dominios o paquetes que confundan, usar el logo en versiones modificadas o distribuir software dañino con el nombre.

### Registro en el SENADI

Primero, la búsqueda fonética de "Ateneo" (USD 16), porque es una palabra común. Después, el registro como marca mixta (nombre y logo) en las clases 9 (software) y 42 (servicios en línea): USD 208, de 5 a 6 meses, sin abogado obligatorio.

### Funciones inspiradas en otros productos

Resaltar, anotar y citar son ideas de uso libre: el derecho de autor andino (Decisión 351, art. 7) protege la forma de expresión y no las ideas, y la Decisión 486 (art. 15) excluye de patentes a los programas como tales. Ateneo no copia el diseño, los íconos, los textos, el código ni los nombres de eLibro u otros productos.

## Comunidad, contribución y sostenibilidad

El repositorio ya cumple todos los estándares de comunidad de GitHub; el sitio del proyecto y las donaciones se activan por etapas, desde la alfa.

### Reglas de contribución

| Regla | Detalle |
|---|---|
| Origen del código | DCO: cada commit firmado con `git commit -s` |
| Mensajes de commit | En inglés, con Conventional Commits |
| Archivos de código | Encabezado SPDX con la licencia |
| Conducta | Contributor Covenant 3.0 |
| Reportes | Plantillas bilingües de error y de mejora |
| Dependencias | Dependabot semanal para cargo, npm y GitHub Actions |
| Documentación | Textos con licencia CC BY-SA 4.0 |

### Sitio del proyecto

Se arma en la fase prototipo, con un dominio sin "-rs" (por ejemplo ateneo.digital, getateneo.org o ateneohq.org). Secciones: inicio con instituciones que lo usan, instalación paso a paso (Docker, npm, pnpm, binario y script), requisitos del servidor medidos en el prototipo, guías, manual del bibliotecario y del administrador, comunidad en GitHub Discussions, hoja de ruta pública, apoyo, seguridad y marca.

### Sostenibilidad

Ateneo es y será siempre gratuito; no se usa el modelo open-core.

| Fuente | Detalle | Cuándo |
|---|---|---|
| Apoyo sin dinero | Estrellas, reportes, traducciones, documentación, difusión y código | Ya publicado en el README |
| GitHub Sponsors | Sin comisión y disponible en Ecuador, con `.github/FUNDING.yml` | Alfa |
| Servicios | Instalación, migración y soporte con contrato | Desde la alfa |
| Open Collective | Comisión del 10 %, cuentas públicas y facturas para instituciones | 1.0 |
| Servicio alojado | Ateneo administrado para quien no tenga servidor | Después de la 1.0 |
| Fondos | `funding.json` y postulación a FLOSS/fund | Con uso real |

La sección de apoyo dice quién recibe el dinero (hoy, una sola persona), en qué se usa y qué no compra una donación: ni prioridad, ni soporte privado, ni decisiones sobre el rumbo. Donar no incluye soporte; la instalación por encargo es un servicio aparte. Desde la 1.0 se publican informes de transparencia cada 3 o 6 meses. Dentro de Ateneo solo hay un enlace discreto en **Sistema → Acerca de**.

## Hoja de ruta

Ateneo avanza en cinco etapas, de la 0.1 a la 1.0, y la alfa (0.2) se probaría en el ISTS (por confirmar).

> **Diagrama · hoja de ruta: 5 etapas, de la 0.1 a la 1.0** ("Cinco etapas llevan a Ateneo de los cimientos a la 1.0").
> Cinco bloques unidos por flechas, de arriba abajo, con la 1.0 resaltada al final. Su contenido es la tabla siguiente.

| Versión | Etapa | Contenido |
|---|---|---|
| 0.1 | Cimientos | Instalación por consola, modelo de datos, cuentas, roles y 2FA. Depósito, sitio público, búsqueda y OAI-PMH |
| 0.2 | Alfa | Importación desde DSpace con redirecciones, visor PDF y citas. Respaldos, logo y pruebas en el ISTS (por confirmar) |
| 0.3 a 0.5 | Beta | OCR, estadísticas, tipografías instalables y páginas legales. Asistente completo y medición de requisitos del servidor |
| 0.6+ | Módulos | Fase 1 de la IA y biblioteca abierta, si la investigación la aprueba. Más tipos de obra y módulos opcionales |
| 1.0 | Producción | Versión estable para cualquier institución. Open Collective e informes de transparencia |

El reparto exacto de funciones entre versiones se ajusta con lo que muestre el prototipo; la versión pública de esta hoja de ruta vivirá en el sitio del proyecto.

---

# Parte 2 — Sitio público

Así se ve y se usa Ateneo desde el lado del lector: sitio público, lectura, citas, cuentas y privacidad.

## Sitio público

Todo lo que publica la institución se busca, lee, descarga y cita sin cuenta, con la identidad visual de cada institución.

> **Mockup · portada de la institución en escritorio y celular** ("La portada lleva la identidad de cada institución").
> En escritorio: cabecera con el logo y el nombre "Instituto Tecnológico Demo", los enlaces Buscar, Colecciones e Iniciar sesión, y una franja de color con "Repositorio Institucional" y el buscador. Debajo, "Últimas obras" a la izquierda y "Colecciones" a la derecha (Software, Enfermería, Contabilidad), con el pie "Hecho con Ateneo" y "Cookies · Cómo usamos tus datos".
> En celular, la misma portada en una columna: siglas "ITD", menú de hamburguesa, buscador y últimas obras.

### Direcciones

| Dirección | Contenido |
|---|---|
| `/` | Portada: buscador, últimas obras y colecciones |
| `/buscar` | Resultados con filtros |
| `/obra/{id}/{titulo}` | Página de la obra |
| `/coleccion/{nombre}` y `/autor/{nombre}` | Obras de una colección o de una persona |
| `/mi-bibliografia` | Lo guardado por el lector |
| `/estadisticas` | Solo si la institución la activa |
| `/acerca`, `/privacidad`, `/cookies`, `/terminos` | Información y páginas legales |
| `/oai`, `/sitemap.xml` | Para cosechadores y buscadores |
| Rutas de DSpace 6 y 8 | Redirección 301 a las nuevas |

### Búsqueda

- Análisis en español: "investigación" también encuentra "investigaciones", sin importar tildes ni mayúsculas.
- Tolera errores de escritura.
- Peso de cada campo: título ×3, autores y palabras clave ×2, resumen ×1,5, texto completo ×1.
- Filtros por origen, tipo, año, carrera, idioma y acceso.

> **Mockup · resultados de búsqueda con filtros** ("La búsqueda filtra por origen, tipo, año y carrera").
> La dirección `/buscar?q=residuos` con la búsqueda "residuos sólidos" y 128 resultados. A la izquierda, filtros: Origen (De la institución, Biblioteca abierta), Tipo (Tesis 84, Artículos 44), un deslizador de año y Carrera (Tecnología Ambiental 52).
> A la derecha, los resultados: "Gestión de residuos sólidos en mercados municipales" (Pérez, A. · 2024 · Tesis · Tecnología Ambiental, con enlaces Citar · PDF) y "Manual de reciclaje para gobiernos locales" (Editorial abierta · 2021 · Libro · CC BY 4.0), marcado con la etiqueta "Biblioteca abierta".

### Página de una obra

Muestra los metadatos, la licencia, el estado, los botones de lectura, descarga, cita y Mi bibliografía, el contador público y las obras relacionadas. Una obra embargada muestra solo los metadatos y la fecha de liberación.

> **Mockup · página de una obra en escritorio y celular** ("La página de la obra reúne lectura, descarga, cita y metadatos").
> En escritorio: la dirección `/obra/1042/gestion-de-residuos`, las migas Inicio › Tecnología Ambiental › Tesis, el título, "Ana Pérez · Tutor: Luis Mora", las etiquetas "Publicada" y "CC BY 4.0", una miniatura y los botones Leer en línea, Descargar PDF, Citar y Mi bibliografía. Abajo van año 2024, tipo "Tesis de tecnología", identificador `ark:/99999/fk4demo1042`, "1.240 visitas · 312 descargas" y "Obras relacionadas".
> En celular, lo mismo en una columna, con Citar y Guardar lado a lado y solo el contador de visitas.

> **Mockup · obra embargada, colección y autor** ("Obra embargada, colección y autor tienen su propia página").
> Tres tarjetas. Obra embargada: "Sistema de riego automatizado", etiqueta "Embargada", "Archivo disponible desde 15 de marzo de 2027" y un recuadro "PDF aún no disponible".
> Colección: "Tecnología Ambiental · Colección · 86 obras" con orden "Más recientes". Autor: "Ana Pérez · 3 obras · ORCID", con avatar y la sección "Como autora y tutora".

## Lectura

Cada obra se lee en un visor propio que funciona en cualquier navegador, con herramientas para estudiar sin crear cuenta.

### Visor PDF

El visor usa PDF.js, así que no depende de Firefox ni de un lector externo.

- Ir a una página con el lugar resaltado.
- **Citar esta página** y **Citar fragmento:** copia el texto entre comillas con su cita y número de página.
- Resaltados y notas en el formato abierto de anotaciones del W3C.
- Modo "solo lectura en línea" cuando la institución no permite descargar una obra.

> **Mockup · visor PDF con resaltado, nota y cita de un fragmento** ("El visor permite resaltar, anotar y citar un fragmento con su página").
> Barra superior con "Volver a la obra", "Página 12 de 84" y los botones Resaltar, Nota, Citar y Fichas. A la izquierda van miniaturas de las páginas y al centro la página con un párrafo seleccionado y un menú flotante "Resaltar · Nota · Citar fragmento".
> A la derecha, una tarjeta "Copiado: "…separar en origen reduce costos…" (Pérez, 2024, p. 12)" y una nota "Comparar con tabla 3" marcada con un punto en el margen.

### Mi bibliografía y fichas de lectura

**Mi bibliografía** guarda las obras elegidas y genera la bibliografía completa en el estilo seleccionado. Exporta a BibTeX, RIS y CSL-JSON, conecta con Zotero y ofrece RSS. **Fichas de lectura** exporta todos los resaltados y notas de una obra con sus citas. Sin sesión todo se guarda en el navegador; con sesión, en la cuenta del lector.

> **Mockup · Mi bibliografía y fichas de lectura** ("Mi bibliografía y las fichas de lectura reúnen lo guardado").
> A la izquierda, "Mi bibliografía · 4 obras · Estilo APA 7" con la lista, los botones "Copiar bibliografía" y "Exportar", y la nota "Guardado en este navegador. Inicia sesión para guardarlo en tu cuenta".
> A la derecha, "Fichas de lectura" de "Gestión de residuos sólidos…" con tres fichas (p. 12, p. 27 con nota y p. 41) y el botón "Exportar fichas con sus citas".

### Accesibilidad

El lector puede cambiar el tamaño de letra, pasar a Atkinson Hyperlegible u OpenDyslexic, ajustar el espaciado entre líneas y activar alto contraste. La elección se guarda solo cuando cambia algo.

> **Mockup · menú de accesibilidad** ("El lector ajusta la lectura a su medida").
> Junto al texto de la obra, un panel "Accesibilidad" con tamaño de letra (A−, A, A+), tipografía (Literata, Atkinson u OpenDyslexic), un deslizador de espaciado entre líneas, un interruptor de alto contraste y la opción "Restablecer".

## Citas

Un solo botón "Citar" genera la referencia en dos clics, en el estilo y el idioma elegidos, con más opciones que Google Scholar y eLibro y sin pedir cuenta.

> **Mockup · ventana Citar en escritorio y celular** ("Citar toma dos clics y genera la referencia en el estilo e idioma elegidos").
> En escritorio, una ventana con pestañas Bibliografía, En el texto y Exportar, selectores de estilo (APA 7.ª edición) e idioma de la cita (Español), y la referencia "Pérez, A. (2024). Gestión de residuos sólidos en mercados municipales [Tesis]. Instituto Tecnológico Demo. https://repositorio.demo.edu.ec/ark:/99999/fk4demo1042" con el botón Copiar. Debajo aparecen MLA e IEEE y "Más estilos: buscar entre miles de estilos CSL".
> En celular, una hoja inferior con el selector APA 7, el botón Copiar y "Otros estilos y exportar".

| Función | Google Scholar | eLibro | Ateneo |
|---|---|---|---|
| Estilos | 5 | Varios | APA 7 (por defecto), MLA, Chicago, Harvard, Vancouver, IEEE, ISO 690 y buscador de estilos CSL |
| Idioma de la cita | Inglés | Español | El que elija el lector |
| Cita en el texto | No | No | Sí, por ejemplo (Pérez, 2024, p. 15) |
| Citar página o fragmento | No | Al copiar | Sí, desde el visor |
| Exportar | BibTeX, EndNote, RefMan, RefWorks | Sí | BibTeX, RIS (EndNote, Mendeley, RefWorks, Zotero) y CSL-JSON |
| Enlace permanente en la cita | No | No | Sí (ARK o handle) |
| Requiere cuenta | No | Sí | No |

### Cómo funciona

- **hayagriva**, una librería de Rust del equipo de Typst (licencia MIT o Apache-2.0), formatea las citas con los estilos del repositorio oficial de CSL y sus archivos de idioma.
- **Estilo e idioma van separados.** El idioma cambia los términos ("y" / "and", "Recuperado de" / "Retrieved from", "s.f." / "n.d."), los meses y las fechas. El título y los autores nunca se traducen; si la obra tiene un título traducido guardado, los estilos que lo piden lo muestran entre corchetes.
- Si un estilo usa siempre un idioma, el selector se desactiva y lo indica.
- El administrador elige la variante del español entre las disponibles en CSL (por ejemplo `es-ES` o `es-MX`). Si falta un idioma, se usa inglés y se avisa.
- La cita APA en el idioma por defecto se guarda ya generada; las demás se generan en el servidor al pedirlas.
- Los estilos y archivos de idioma de CSL tienen licencia CC BY-SA 3.0: se acredita al proyecto CSL en **Sistema → Acerca de → Créditos**.

## Cuentas, privacidad y cookies

Hay dos tipos de cuenta, nada se bloquea a quien no inicia sesión y, con la configuración normal, no hace falta ventana de cookies.

### Tipos de cuenta

| Tipo | Quién | Cómo se crea | Qué ve |
|---|---|---|---|
| Personal | Bibliotecarios, revisores y administradores | La registra el administrador | El panel, según su rol |
| Lector | Estudiantes y docentes con correo institucional | Se crea sola al entrar con Google o Microsoft, si el dominio está permitido | El sitio público y sus datos; nunca el panel |

El inicio de sesión usa una aplicación de Google o Microsoft que registra cada institución con ayuda del asistente (las direcciones de retorno dependen del dominio). Se admiten varios dominios, y los datos de la cuenta se guardan en Ateneo, sin depender de Google ni de Microsoft.

| Función | Sin sesión | Con sesión de lector |
|---|---|---|
| Leer, descargar y citar lo publicado | Sí | Sí |
| Notas, resaltados y Mi bibliografía | En el navegador | En la cuenta, en cualquier dispositivo |
| Biblioteca abierta (si se activa) | Pide iniciar sesión | Sí |
| IA conversacional (si se activa) | No, o un modo de prueba limitado | Sí |

> **Mockup · inicio de sesión y paso de datos del navegador a la cuenta** ("El lector entra con su cuenta institucional y conserva lo guardado").
> A la izquierda, la pantalla de entrada de "Instituto Tecnológico Demo": "Inicia sesión con tu cuenta institucional", con los botones "Continuar con Google" y "Continuar con Microsoft" y la nota "Solo correos @demo.edu.ec".
> A la derecha, el aviso "Encontramos datos en este navegador: 12 citas · 5 notas · 3 resaltados. ¿Pasarlos a tu cuenta para verlos en cualquier dispositivo?" con los botones "Pasar a mi cuenta" y "Ahora no".

### Cuentas para toda la vida

- Las cuentas de lector **nunca se borran solas**, ni al graduarse: los correos institucionales siguen activos de por vida. Como respaldo, el lector puede **vincular otro acceso** (su Google o Microsoft personal).
- **Eliminación solo por trámite:** el botón "Solicitar eliminación de mis datos" crea una solicitud que el super admin ejecuta; la auditoría guarda solo la fecha y el motivo.
- Cuando alguien del personal deja la institución, su cuenta se **desactiva** y sus depósitos conservan su nombre; borrarla del todo es decisión del administrador.
- En **Mis datos**, cada lector descarga todo lo suyo. Sus notas son privadas: ni el administrador las lee.

> **Mockup · Mis datos y aviso de privacidad** ("Mis datos permite descargar todo, vincular otro acceso y pedir la eliminación").
> A la izquierda, "Mis datos": correo institucional ana.perez@demo.edu.ec, accesos vinculados (Google institucional) con "+ Vincular otro acceso (Google o Microsoft personal)", el botón "Descargar todos mis datos" y, en rojo, "Solicitar eliminación de mis datos", con la nota "La solicitud la revisa el super admin como trámite".
> A la derecha, un "Aviso de privacidad" breve ("Guardamos tu nombre y correo. Guardamos tus citas, notas y resaltados; solo tú los ves. Nunca vendemos ni compartimos tus datos.") con los botones "Entendido" y "Ver la política completa".

### Cookies y almacenamiento

Se sigue el estándar europeo, porque la ley ecuatoriana de protección de datos (LOPDP) se basa en él y no tiene una norma propia sobre cookies.

| Qué se guarda | Dónde | ¿Pide consentimiento? |
|---|---|---|
| Sesión y protección CSRF | Cookie | No: es estrictamente necesaria |
| Bibliografía, notas y resaltados sin sesión | Navegador, solo al presionar "Guardar" | No: es el servicio que el lector pide |
| Preferencias de accesibilidad y estilo de cita | Navegador, solo cuando el lector cambia algo | Se explica en el aviso |
| Estadísticas y control de bots | Servidor | No usa cookies |
| Tipografías | Servidor de la institución | El navegador nunca conecta con Google |

El resultado es solo una nota en el pie de página. Si la institución agrega algo de terceros (por ejemplo Google Analytics en "HTML personalizado"), aparece un aviso **Aceptar / Rechazar** con los dos botones igual de visibles, nada marcado de antemano, el servicio bloqueado hasta aceptar, un enlace fijo para cambiar la decisión y el registro anónimo del consentimiento. Rechazar nunca quita funciones: un "muro de cookies" no es consentimiento válido y la LOPDP (art. 8) exige que sea libre.

La página **"Cómo usamos tus datos"** explica qué se mide y para qué, y ofrece el interruptor **"No contar mis visitas"**. Ateneo respeta además la señal Global Privacy Control del navegador.

> **Mockup · Cómo usamos tus datos** ("Una página explica qué se mide y permite no ser contado").
> El título "Cómo usamos tus datos" y el subtítulo "Sin cookies de seguimiento ni servicios de terceros", con tres columnas: Qué medimos (de qué sitio llegas, tipo de dispositivo, país si está activado), Para qué (mejorar el repositorio, mostrar el impacto de cada obra) y Qué no hacemos (no vendemos datos, no usamos rastreadores, no seguimos a nadie).
> Abajo, el interruptor "No contar mis visitas" con la nota "También respetamos la señal Global Privacy Control".

> **Mockup · política de cookies y aviso Aceptar / Rechazar** ("La política de cookies se genera sola y el aviso solo aparece con terceros").
> Arriba, la "Política de cookies" con una tabla de nombre, uso y duración: `ateneo_sesion` (mantener la sesión iniciada, hasta cerrar sesión), `ateneo_csrf` (seguridad de los formularios, sesión) y Preferencias (accesibilidad y estilo de cita, hasta borrarlas), y la nota "Este sitio solo usa cookies necesarias".
> Abajo, un aviso de ejemplo "Este sitio usa Google Analytics, agregado por la institución" con los botones Rechazar y Aceptar del mismo tamaño; la nota aclara que el aviso solo aparece si la institución agrega servicios de terceros.

### Páginas legales

Se generan con los datos del paso "Datos legales" y con los módulos activos: política de privacidad, política de cookies (con la tabla de lo que el sitio realmente usa), términos de uso, derechos de autor y retiro de contenido, y declaración de accesibilidad. Se editan en **Sistema → Páginas legales**, guardan historial y muestran "Plantilla" hasta que alguien las confirme. Cuando cambia la política de privacidad, se avisa a los lectores en su siguiente inicio de sesión.

> **Mockup · editor de páginas legales** ("Las páginas legales se editan con aviso de plantilla e historial").
> A la izquierda, la lista de páginas: Política de privacidad (seleccionada), Política de cookies, Términos de uso, Derechos de autor y retiro, y Accesibilidad.
> A la derecha, el aviso amarillo "Plantilla: revísala con tu asesor legal antes de confirmarla", un editor con negrita, cursiva, enlace y lista, y abajo "Versión 3 · Historial de cambios" con el botón "Confirmar revisión".

---

# Parte 3 — Panel

Así trabaja el personal en el panel de Ateneo, y lo que queda para después.

## Depósito, revisión y OCR

Solo el personal sube obras, con un asistente de 4 pasos que se autollena con los datos internos del PDF.

### Depósito

1. **Archivo:** Ateneo lee los metadatos del PDF (título, autor, fecha, idioma) y llena lo que puede; si el PDF es escaneado, lo envía a OCR.
2. **Metadatos:** el formulario del tipo de obra, con listas controladas y personas sin duplicar.
3. **Licencia y acceso:** licencia del catálogo SPDX completo, estado y opción de solo lectura en línea.
4. **Revisión:** resumen antes de enviar y portada opcional con las plantillas de la institución.

> **Mockup · depósito, paso 1: subir y autollenar** ("Al subir el PDF, Ateneo llena lo que puede y detecta si es escaneado").
> Las migas Panel › Obras › Depositar y una barra de 4 pasos (Archivo, Metadatos, Licencia y acceso, Revisión). A la izquierda, el archivo subido "tesis-ana-perez.pdf · 2,4 MB · 84 páginas" con la barra de carga completa.
> A la derecha, "Autollenado desde el PDF" con marcas verdes en título, autor (Ana Pérez), año (2024) e idioma (Español), el aviso "PDF escaneado: se enviará a OCR al guardar" y el botón Siguiente.

> **Mockup · depósito, pasos 2 a 4** ("Metadatos, licencia y acceso se completan antes de enviar a revisión").
> Tres columnas. Metadatos: título, autores (Ana Pérez), tutor (Luis Mora) y palabras clave de lista controlada (reciclaje, mercados). Licencia y acceso: licencia del catálogo SPDX (CC BY 4.0), estado (Publicar, Embargar hasta una fecha, Ocultar con motivo obligatorio) y la casilla "Solo lectura en línea".
> Revisión: el resumen, la portada con "Plantilla institucional" y el botón "Enviar a revisión".

El tipo de obra por defecto es **Artículo**; los demás (tesis, libros, ensayos, informes) se activan desde el panel, cada uno con su propio formulario.

### Estados de una obra

| Estado | Quién la ve | Uso |
|---|---|---|
| Publicada | Todos | Estado normal |
| Embargada | Todos ven los metadatos; el archivo se libera en la fecha fijada | Patentes o publicación pendiente en una revista |
| Oculta | Solo el personal | "No publicar de momento"; exige escribir el motivo, como pide la LOES |

No existe un acceso "restringido" permanente: se reemplazó por el estado Oculta.

> **Mockup · lista de obras por estado** ("Las obras se organizan por estado: por revisar, publicadas, embargadas y ocultas").
> Pestañas Por revisar (4), Publicadas (1.180), Embargadas (12) y Ocultas (3), con un buscador. La tabla tiene columnas Título, Autor, Tipo, Estado y Fecha, y filas en estado Por revisar (hoy, ayer), Devuelta (22/11) y Corregida (20/11).

### Revisión

El revisor aprueba, devuelve con comentarios o corrige. Cada cambio guarda una foto completa de la obra, así que cualquier versión se puede ver y restaurar. Las notificaciones son internas, dentro del panel.

> **Mockup · revisión de una obra e historial** ("El revisor aprueba o devuelve con comentarios y puede restaurar versiones").
> A la izquierda, la vista previa del documento. A la derecha, la lista "Revisión" con marcas verdes (Metadatos completos, Licencia y autorización, Texto buscable con OCR listo), un campo "Comentario para el depositante" y los botones "Aprobar y publicar" y "Devolver".
> Debajo, el "Historial de versiones": v3 hoy de Luis Mora, y v2 de ayer y v1 del 20/11 de Ana Pérez, cada una con "Ver · Restaurar".

### Colecciones y personas

Las colecciones forman un árbol de comunidades y subcolecciones. Los autores y tutores se guardan una sola vez; Ateneo detecta el mismo nombre escrito de varias formas y permite unirlos.

> **Mockup · árbol de colecciones y unión de autores** ("Las colecciones forman un árbol y los autores duplicados se unen en uno").
> A la izquierda, el árbol "Instituto Tecnológico Demo" con Tecnología Ambiental (86), Software (240, con subcolecciones Tesis y Artículos) y Enfermería (130), más "+ Nueva colección".
> A la derecha, "Posibles duplicados · Mismo nombre escrito de varias formas": Ana Pérez (3 obras), Pérez, Ana (1 obra) y A. Pérez (1 obra), con el botón "Unir en una sola persona".

### OCR (texto de PDF escaneados)

| Pieza | Tipo | Función |
|---|---|---|
| ocr-rs (PaddleOCR) | Incluido | Reconoce el texto de páginas escaneadas |
| Capa de texto invisible (lopdf) | Incluido | Hace el PDF buscable y seleccionable |
| OCRmyPDF | Opcional | OCR de mayor calidad si está instalado |
| veraPDF | Opcional | Valida PDF/A para preservación |
| Cola de tareas | Incluida | Procesa en segundo plano y muestra el avance |

> **Mockup · cola de OCR y tareas** ("La cola de tareas muestra el avance de OCR, índices, respaldos e importaciones").
> Una tabla con columnas Tarea, Detalle, Estado y Avance: OCR de "Sistema de riego automatizado" (en proceso), Importación "DSpace, lote 3 de 10" (en proceso), Índice "Reconstrucción del buscador" (en cola) y Respaldo "Diario, 02:00" (completado, barra verde llena).

## Administración, roles y seguridad

El super admin puede hacer todo y delegar cualquier función asignando roles; la seguridad sigue el OWASP Top 10 de 2025.

### Panel

Cada persona solo ve los menús de su rol. El Inicio muestra los pendientes de configuración y la salud del sistema: base de datos, disco, índice, cola de tareas, último respaldo y módulos.

| Menú | Contenido |
|---|---|
| Obras, Colecciones, Personas | Trabajo diario (ver Depósito) |
| Formularios | Campos, listas controladas y mapeos (ver Metadatos) |
| Usuarios y roles | Cuentas y roles |
| Apariencia | Colores, logo, tipografías y plantillas de portada |
| Estadísticas, Módulos, Respaldos, Seguridad | Ver sus secciones |
| Sistema | Institución, páginas legales, tareas, registros, actualizaciones, instrucciones manuales y Acerca de |

> **Mockup · inicio del panel y salud del sistema** ("El inicio del panel muestra pendientes y la salud del sistema").
> Menú lateral con Inicio, Obras, Colecciones, Personas, Formularios, Usuarios y roles, Apariencia, Estadísticas, Módulos, Respaldos, Seguridad y Sistema. Arriba, un recuadro amarillo "Pendientes de configuración: Falta el logo · Inicio de sesión con Google o Microsoft sin configurar".
> Cuatro tarjetas de salud: Base de datos (En orden), Espacio en disco (62 % usado), Último respaldo (Hoy, 02:00) y Cola de tareas (2 en proceso), más "Por revisar: 4 obras esperan revisión" y "Avisos: Nueva versión disponible".

### Roles

Hay roles predefinidos (super admin, administrador, depositante, revisor, editor de metadatos) y **roles personalizados** armados permiso por permiso. El super admin conserva siempre todos los permisos. Los roles son solo para el personal; los lectores no tienen roles.

> **Mockup · usuarios, roles y rol personalizado** ("Cada persona tiene un rol y los roles personalizados se arman permiso por permiso").
> A la izquierda, "Usuarios y roles" con columnas Persona, Rol y Estado: Ana Torres (Super admin, Activa), Luis Mora (Revisor, Activa), Carla Ruiz (Catalogador, Activa) y Pedro Vega (Depositante, Desactivada), más "+ Agregar persona".
> A la derecha, el rol personalizado "Catalogador" con casillas: marcadas Depositar obras, Editar metadatos y Ver estadísticas; sin marcar Publicar obras, Administrar usuarios y Cambiar la apariencia; y el botón "Guardar rol".

### Contraseñas y verificación en dos pasos

- Contraseñas guardadas con **Argon2id**, de **15 caracteres como mínimo siempre**, con lista de contraseñas prohibidas.
- **2FA opcional y recomendado:** se usa una app de autenticación aparte (TOTP), independiente de la cuenta institucional, con 10 códigos de respaldo. Al activarlo aparecen el código QR, la clave manual y la verificación.

> **Mockup · activar la verificación en dos pasos** ("Activar 2FA muestra el código QR, la clave manual y 10 códigos de respaldo").
> A la izquierda, "Verificación en dos pasos": 1. escanear el código QR con la app de autenticación (con la clave manual "JBSW Y3DP EHPK 3PXP" para quien no pueda escanear) y 2. escribir el código de 6 dígitos en seis casillas, con el botón Verificar.
> A la derecha, "Códigos de respaldo · Úsalos si pierdes el celular": 10 códigos de 8 dígitos en formato 0000-0000, con los botones Descargar y Copiar.

### Protección contra bots y descargas masivas

Límites por IP y por sesión con detección de descargas masivas, bloqueo escalonado (15 minutos, 1 hora, 24 horas), redes de confianza que nunca se bloquean y bots buenos permitidos (buscadores y cosechadores). Toda acción administrativa queda en el registro de auditoría.

> **Mockup · seguridad: bloqueos, redes de confianza y bots** ("Seguridad muestra los bloqueos, las redes de confianza y los bots permitidos").
> "IPs bloqueadas" con la nota "Bloqueo escalonado: 15 min → 1 h → 24 h": 181.112.0.0 por descargas masivas (quedan 52 min, nivel 2) y 200.7.0.0 por demasiadas solicitudes (quedan 9 min, nivel 1), cada una con "Desbloquear".
> "Redes de confianza": 10.0.0.0/8 (red interna de la institución), con "+ Agregar red". "Bots permitidos": Googlebot, Bingbot, La Referencia y OpenAIRE.

### Módulos

Cada módulo se activa con un interruptor; si depende de otro, Ateneo lo avisa antes.

> **Mockup · módulos** ("Los módulos se activan o desactivan con un interruptor").
> Seis tarjetas con interruptor. Activados: Estadísticas (visitas y descargas propias), OCR (texto de PDF escaneados) y SWORD (depósito desde otros sistemas). Desactivados: Ranking público (obras más leídas), Biblioteca abierta (en investigación) y Asistente de IA (futuro, versión 0.6 o posterior).
> Nota al pie: "Si un módulo necesita otro, Ateneo lo indica antes de activarlo".

### Reporte de vulnerabilidades

Por reporte privado en GitHub (`SECURITY.md`): respuesta en 7 días, evaluación en 14 y corrección en hasta 90, con protección legal para quien reporte de buena fe.

## Metadatos, identificadores y cosecha

Cada campo se publica en tres formatos a la vez para que Google Scholar, los buscadores y los cosechadores lo entiendan, y las direcciones antiguas de DSpace siguen funcionando.

| Salida | Para quién | Ejemplos |
|---|---|---|
| Etiquetas `citation_*` | Google Scholar | `citation_title`, `citation_author`, `citation_pdf_url` |
| schema.org en JSON-LD | Google y otros buscadores | `ScholarlyArticle`, `Thesis` |
| Dublin Core / OpenAIRE | Cosechadores OAI-PMH (La Referencia, OpenAIRE) | `dc.title`, `dc.contributor.advisor` |

### Editor de formularios

Cada tipo de obra tiene su formulario. Cada campo lleva su etiqueta visible, su tipo y sus mapeos, con todas las opciones de mapeo disponibles. Los valores repetidos usan **listas controladas** y las personas se guardan una sola vez.

> **Mockup · editor de formularios y mapeos** ("Cada campo tiene su etiqueta y sus mapeos a Dublin Core, schema.org y Google Scholar").
> A la izquierda, "Formulario: Tesis" con campos que se arrastran para ordenar: Título (obligatorio), Autores (obligatorio), Tutor (seleccionado), Carrera (lista controlada), Palabras clave y Resumen, más "+ Agregar campo".
> A la derecha, "Campo: Tutor" con etiqueta visible "Tutor", tipo "Persona" y mapeos: Dublin Core / OpenAIRE → `dc.contributor.advisor`, schema.org → contributor (tutor) y Google Scholar → No aplica. Tiene además un interruptor "Obligatorio" apagado y el botón Guardar.

### Identificadores y direcciones

| Elemento | Decisión |
|---|---|
| Dirección de cada obra | `/obra/{id}/{titulo-en-slug}` |
| Identificador persistente | ARK, además del handle anterior, cuyo prefijo se conserva |
| Rutas de DSpace 6 y 8 | Redirección 301 automática |
| Identificadores OAI | Se conservan los originales y se atienden las rutas OAI antiguas, para no duplicar registros |

### Protocolos

OAI-PMH para cosecha, IIIF para imágenes, SWORD para depositar desde otros sistemas y sitemap para buscadores.

## Apariencia y tipografía

La tipografía por defecto es Literata con Atkinson Hyperlegible, y cada institución puede instalar otras fuentes, revisadas automáticamente para evitar problemas de licencia.

| Uso | Tipografía por defecto |
|---|---|
| Títulos y texto de lectura | Literata |
| Interfaz | Atkinson Hyperlegible Next |
| Código e identificadores | Atkinson Hyperlegible Mono |

En **Apariencia** se cambian además los colores, el logo y las plantillas de portada de las obras.

### Fuentes instalables

Se pueden instalar fuentes de Fontsource o Google Fonts (se descargan al servidor) o subir una fuente comprada. Toda fuente subida pasa por este análisis:

1. **Limpieza** con OTS (OpenType Sanitizer), para bloquear archivos maliciosos.
2. **Lectura** con skrifa / read-fonts: nombre, permisos de incrustación (`fsType`), fabricante (`achVendID`), caracteres incluidos y glifos duplicados.
3. **Semáforo:** rojo bloquea (por ejemplo, una versión de prueba o sin permiso de uso web), amarillo pide la prueba de compra de la licencia y verde permite instalar.
4. **Optimización:** se convierte a WOFF2 con solo los caracteres necesarios.

Así ninguna institución usa, sin querer o a propósito, una versión de prueba o una fuente sin licencia.

> **Mockup · apariencia y semáforo de tipografías** ("Cada fuente subida pasa por un semáforo de licencias antes de instalarse").
> A la izquierda, "Apariencia": tres colores (azul, naranja y verde), una zona "Cambiar logo" y las tipografías en uso (títulos y lectura: Literata; interfaz: Atkinson Hyperlegible Next; código: Atkinson Hyperlegible Mono).
> A la derecha, "Instalar tipografía" con pestañas Fontsource, Google Fonts y Subir archivo: MiFuente-Trial.otf en rojo ("Versión de prueba: bloqueada"), Corporativa.woff2 en amarillo (botón "Subir prueba" de compra de la licencia) y Merriweather (Google Fonts) en verde ("Licencia libre: lista para instalar", botón Instalar). Al pie: "Se convierte a WOFF2 y se sirve desde el servidor".

## Estadísticas

Las estadísticas son propias, se calculan en el servidor sin cookies ni terceros y siguen el estándar COUNTER de las bibliotecas.

| Pestaña | Qué muestra |
|---|---|
| Resumen | Visitas y descargas por día, semana, mes y año |
| Procedencia | Google, Google Scholar, redes sociales, La Referencia, enlace directo y otros sitios |
| Dispositivos | Celular, computadora o tableta; navegador y sistema |
| Países | Mapa y lista, solo si se activa |
| Obras | Las más leídas y descargadas por colección, carrera y año |
| Búsquedas | Lo más buscado y las búsquedas sin resultados, que indican qué temas faltan |

> **Mockup · ventana de estadísticas del panel (cifras de ejemplo)** ("La ventana de estadísticas muestra de dónde llegan y con qué dispositivo").
> Pestañas Resumen, Procedencia (activa), Dispositivos, Países, Obras, Búsquedas y Opciones, con el periodo "Últimos 30 días". "De dónde llegan", en barras: Google 48 %, Google Scholar 21 %, Enlace directo 14 %, WhatsApp 8 %, Facebook 5 % y La Referencia 4 %.
> A la derecha, "Dispositivos" en una barra apilada (celular 62 %, computadora 35 %, tableta 3 %) y "Búsquedas sin resultados" (energía solar rural, drones agrícolas, contabilidad de costos).

### Opciones (solo el super admin)

| Opción | Por defecto |
|---|---|
| Contador público en cada obra | Activado |
| Estadísticas por país | Desactivado |
| Ranking público de las más leídas | Desactivado |
| Página pública `/estadisticas` | Desactivado |
| No contar al personal con sesión | Activado |
| No contar bots | Siempre |
| Conservación de datos detallados | 12 meses; luego solo totales |

> **Mockup · página pública de estadísticas (cifras de ejemplo)** ("La página pública de estadísticas es opcional y muestra el impacto del repositorio").
> "Estadísticas del repositorio · 2026" con tres cifras grandes: 1.180 obras publicadas, 84.300 visitas y 21.400 descargas.
> Debajo, "Más leídas (ranking opcional)": 1. Gestión de residuos sólidos en mercados municipales (1.240), 2. Aplicación móvil para turnos médicos (980) y 3. Costos de producción en pequeñas cooperativas (860).

### Diferencia con Google Analytics

La procedencia y el dispositivo salen de los datos que el navegador ya envía en cada visita, y la IP solo se usa para el país y se guarda recortada. Ateneo no da el recorrido detallado de cada persona, porque eso exige rastrearla. Si una institución quiere Google Analytics, puede agregarlo con el aviso Aceptar / Rechazar.

## Respaldos, exportación y migración

Los respaldos se programan desde la interfaz, van cifrados y siguen la regla 3-2-1; ninguna institución queda atrapada en Ateneo, ni al entrar ni al salir.

### Respaldos

| Elemento | Decisión |
|---|---|
| Base de datos | `pg_dump` comprimido con zstd |
| Cifrado | age |
| Destinos | Local, S3 y compatibles, Google Drive y OneDrive, mediante OpenDAL; Google Drive y OneDrive se conectan con un clic en su ícono |
| Copias inmutables | S3 Object Lock, para que nadie pueda borrarlas |
| Regla | 3-2-1: tres copias, en dos medios, una fuera de la institución |
| Prueba | Botón "Probar restauración" |

> **Mockup · respaldos** ("Los respaldos se programan desde la interfaz y van a varios destinos cifrados").
> A la izquierda, "Programación" (Diario, 02:00) y "Destinos": Disco local, Google Drive (repositorio@demo.edu.ec) y S3 con Object Lock (inmutable) activos en verde, y OneDrive sin conectar con el botón "Conectar". Al pie: "Cifrado con age · compresión zstd".
> A la derecha, el "Historial" con la etiqueta "Regla 3-2-1 cumplida": hoy 02:00 (1,2 GB), ayer 02:00 (1,2 GB) y 25/11 02:00 (1,1 GB), todos correctos, con los botones "Respaldar ahora" y "Probar restauración".

### Exportación completa

Toda la información se exporta en formatos abiertos: metadatos en JSON y CSV, y archivos en OCFL.

### Migración desde DSpace

Importa comunidades, colecciones, personas, obras y archivos desde DSpace 6 y 8, con pantalla de avance y registro de cada importación. Conserva el prefijo handle y los identificadores OAI, y crea las redirecciones 301 de todas las rutas antiguas.

> **Mockup · importación desde DSpace** ("La importación desde DSpace muestra el avance y conserva handles e identificadores").
> El campo con la dirección https://dspace.demo.edu.ec y la etiqueta "DSpace 8 detectado". Barras de avance: comunidades y colecciones 12 de 12, personas 830 de 830 (ambas completas, en verde), obras 640 de 1.180 y archivos 2,1 de 3,9 GB (en azul, a medias), con "Redirecciones 301 creadas: 640".
> Nota al pie: "Se conservan el prefijo handle y los identificadores OAI originales".

### Actualizaciones

**Sistema → Actualizaciones** avisa cuando hay una versión nueva y muestra el comando según el método de instalación.

## Lo que queda para después

Estos temas no entran en la base inicial; la investigación pendiente se hace cuando la base y lo prioritario estén construidos.

| Tema | Qué falta | Cuándo | Depende de |
|---|---|---|---|
| Requisitos del servidor | Medir RAM, CPU y disco reales | Prototipo (0.2 a 0.5) | Prototipo con datos de prueba |
| Dominio y sitio del proyecto | Elegir dominio y armar el sitio | Prototipo; se publica en la alfa | — |
| Marca en el SENADI | Búsqueda fonética y registro | Prototipo; registro junto con el logo | Logo |
| Logo | Diseñarlo | Antes de la alfa | — |
| GitHub Sponsors y `FUNDING.yml` | Activarlos | Se analizan en el prototipo; se activan en la alfa | Dominio |
| Citas en español | Probar hayagriva en español y elegir la variante regional | Prototipo | — |
| Política "No entrenar IA" | Definir cómo se declara (robots.txt, metadatos, página); Ateneo nunca entrena modelos con las obras | Prototipo | — |
| Biblioteca abierta | Investigar si es factible | Después de la base; módulo 0.6+ solo si es viable | Investigación |
| Asistente de IA | Medir rendimiento y elegir cómo guardar los vectores (pgvector u otra opción en Rust) | Base preparada en 0.1; fase 1 en 0.6+ | Mediciones del prototipo |
| Respaldos incrementales | Integrar rustic_core | Después de la 1.0 | Uso real |
| Extensiones de la comunidad | Sistema en WebAssembly | Después de la 1.0 | API estable |

### Biblioteca abierta (en investigación)

La idea es sumar libros académicos de acceso abierto para leerlos en el visor de Ateneo, sin cuentas en otros sitios. Se descartó pagar a proveedores comerciales (eLibro, EBSCO, ProQuest), porque solo permiten leer en su plataforma y con su cuenta.

| Fuente | Qué aporta | Estado |
|---|---|---|
| DOAB | Catálogo mundial de libros abiertos revisados por pares; metadatos CC0 por OAI-PMH | Confirmado |
| OAPEN | Los PDF, por API REST y OAI-PMH | Confirmado |
| SciELO Libros | Libros iberoamericanos, muchos en español | Falta verificar su API |
| CLACSO | Ciencias sociales de América Latina | Falta verificar licencias |
| Project Gutenberg | Clásicos de dominio público, por catálogo descargable y espejos | Solo autores fallecidos hace más de 70 años |

- **Según la licencia:** CC BY, BY-SA y BY-NC se guardan y muestran; BY-ND se guardan sin modificar el PDF (su texto va aparte); "solo uso personal" no se importa.
- **Modos:** bajo demanda (el PDF se descarga la primera vez que alguien lo abre) o copia completa de una lista elegida.
- **Separada de la producción propia:** atribución visible, citas a la editorial original, sin salida por OAI-PMH, sin etiquetas `citation_*`, enlace canónico a la fuente y estadísticas aparte.
- Si se activa, pide iniciar sesión solo para la biblioteca, y el PDF descargado va siempre limpio, sin DRM.
- **Falta investigar:** factibilidad, API de SciELO, licencias de CLACSO, espacio en disco y confirmar que pedir sesión no contradice las licencias Creative Commons. Si no es factible, no se agrega.

> **Mockup · biblioteca abierta (en investigación)** ("La biblioteca abierta elegiría fuentes, idioma y licencias antes de importar").
> Con la etiqueta "En investigación", tres columnas: Fuentes (DOAB y OAPEN y SciELO Libros marcadas; CLACSO y Project Gutenberg sin marcar), Filtros (idioma español; temas ambiente, software y salud; licencias CC BY, BY-SA, BY-NC y BY-ND) y Modo (Bajo demanda elegido, o Copia completa).
> Abajo, "Vista previa: 1.240 libros encontrados · Unos 4 GB si se copian todos · los de "solo uso personal" se excluyen", con el botón Importar.

### Asistente de IA (futuro)

| Fase | Qué incluye | Quién la usa | Versión |
|---|---|---|---|
| Base | Módulo vacío, campos de sugerencias y texto de cada PDF guardado | — | 0.1 |
| Fase 1 | Búsqueda por significado y sugerencias en el depósito | Todos y el personal | 0.6+ |
| Fase 2 | "Pregúntale a este documento" | Lectores con sesión | Después |
| Fase 3 | Chat del repositorio completo | Lectores con sesión | Después |

- **Sin sesión** se ve lo que la IA prepara al publicar (resumen sencillo revisado por el bibliotecario, obras relacionadas, palabras clave, textos alternativos de imágenes) y la búsqueda por significado.
- **Motor:** local con mistral.rs (Rust, MIT) o un proveedor externo con la clave de la institución, ambos por la misma API compatible con OpenAI.
- **Reglas:** desactivado por defecto; nunca usa obras embargadas u ocultas; siempre cita fuente y página; no entrena modelos; no escribe trabajos por el estudiante; límite de uso por persona; no guarda conversaciones por defecto.

> **Mockup · IA en el visor y sus ajustes (futuro)** ("La IA respondería solo con el documento y citando la página").
> A la izquierda, junto al texto de la obra, el panel "Pregúntale a este documento" con la pregunta "¿Qué metodología usó?", una respuesta con enlaces a las páginas 14 y 15, el aviso "Respuesta generada por IA: verifícala en el documento" y el campo "Escribe tu pregunta…".
> A la derecha, los ajustes del "Asistente de IA" con la etiqueta "Futuro": motor local (mistral.rs, elegido) o proveedor externo, límite de 20 preguntas al día, "Solo con sesión iniciada" activado, "Guardar conversaciones" desactivado y "Usar obras embargadas: Nunca".
