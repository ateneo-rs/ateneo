# Ateneo
Repositorio institucional en Rust, alternativa a DSpace. Licencia AGPL-3.0-or-later.

## Reglas
- Código sin comentarios de ningún tipo.
- Mensajes de commit siempre en inglés, con Conventional Commits y firmados con git commit -s.
- No agregar la línea Co-Authored-By en los commits.
- Cada archivo de código lleva el encabezado SPDX: AGPL-3.0-or-later.
- Responde y explica en español sencillo.
- Antes de planificar o construir, consulta docs/planificacion.md (y los PDF de docs/planificacion/ para ver los mockups).

## Stack
Rust (edición 2024), Axum, sqlx con PostgreSQL, Tantivy, minijinja para el sitio público,
React + MUI para el panel, un solo binario, cliclack para la instalación por consola.

## Ahora
Versión 0.1 (Cimientos): estructura del proyecto, instalación por consola,
modelo de datos, cuentas, roles y 2FA, depósito, sitio público, búsqueda y OAI-PMH.
