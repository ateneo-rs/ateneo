-- SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE settings (
    key text PRIMARY KEY CHECK (key <> ''),
    value jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE audit_log (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    occurred_at timestamptz NOT NULL DEFAULT now(),
    actor_kind text NOT NULL CHECK (actor_kind IN ('user', 'console', 'system')),
    actor_id uuid,
    action text NOT NULL CHECK (action <> ''),
    target_type text,
    target_id text,
    details jsonb NOT NULL DEFAULT '{}'::jsonb,
    ip inet
);

CREATE INDEX audit_log_occurred_at_idx ON audit_log (occurred_at DESC);
CREATE INDEX audit_log_actor_idx ON audit_log (actor_id, occurred_at DESC);
CREATE INDEX audit_log_target_idx ON audit_log (target_type, target_id);
