# Ateneo
Repositorio institucional en Rust, alternativa a DSpace. Licencia AGPL-3.0-or-later.

## Reglas
- Código sin comentarios de ningún tipo. La única excepción es la línea SPDX del encabezado, con la sintaxis de comentario de cada lenguaje.
- Mensajes de commit siempre en inglés, con Conventional Commits y firmados con git commit -s.
- No agregar la línea Co-Authored-By en los commits.
- Cada archivo de código lleva el encabezado SPDX: AGPL-3.0-or-later.
- Responde y explica en español sencillo.
- Antes de construir algo, lee solo la sección de docs/planificacion.md y de docs/arquitectura-0.1.md que corresponda a la tarea. Abre los PDF solo si necesitas ver un mockup.
- La estructura y el orden de la 0.1 están en docs/arquitectura-0.1.md.

## Stack
Rust (edición 2024), Axum, sqlx con PostgreSQL, Tantivy, minijinja para el sitio público,
React + MUI para el panel, un solo binario, cliclack para la instalación por consola.

## Ahora
Versión 0.1 (Cimientos): estructura del proyecto, instalación por consola,
modelo de datos, cuentas del personal, roles y 2FA, depósito con ARK, sitio público,
búsqueda, OAI-PMH y backup now (local y sin cifrar, solo para pruebas).
Google/Microsoft y las cuentas de lector pasan a la 0.2.
