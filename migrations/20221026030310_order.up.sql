CREATE TYPE rsvt.reservation_status AS ENUM ('unknown', 'pending', 'confirmed', 'blocked');
CREATE TYPE rsvt.reservation_update_type AS ENUM ('unknown', 'create', 'update', 'delete');

CREATE TABLE rsvt.reservations (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    user_id VARCHAR(64) NOT NULL,
    rstatus rsvt.reservation_status NOT NULL DEFAULT 'pending',
    resource_id VARCHAR(64) NOT NULL,
    rperiod TSTZRANGE NOT NULL,
    note TEXT,

    CONSTRAINT reservations_pkey PRIMARY KEY (id),
    CONSTRAINT reservations_conflict EXCLUDE USING gist (
        resource_id WITH =,
        rperiod WITH &&
    )
);

CREATE INDEX reservations_resource_id_idx ON rsvt.reservations (resource_id);
CREATE INDEX reservations_user_id_idx ON rsvt.reservations (user_id);
