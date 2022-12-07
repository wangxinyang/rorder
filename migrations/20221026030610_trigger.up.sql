-- resevation change queue
CREATE TABLE rsvt.reservation_changes (
    id SERIAL NOT NULL,
    reservation_id bigserial NOT NULL,
    op rsvt.reservation_update_type NOT NULL,
    CONSTRAINT reservation_changes_pkey PRIMARY KEY (id)
);
CREATE INDEX reservation_changes_reservation_id_op_idx ON rsvt.reservation_changes (reservation_id, op);

-- trigger for add/update/delete a reservation
CREATE OR REPLACE FUNCTION rsvt.reservations_trigger() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- update reservation_changes
        INSERT INTO rsvt.reservation_changes (reservation_id, op) VALUES (NEW.id, 'create');
    ELSIF TG_OP = 'UPDATE' THEN
        -- if status changed, update reservation_changes
        IF OLD.rstatus <> NEW.rstatus THEN
            INSERT INTO rsvt.reservation_changes (reservation_id, op) VALUES (NEW.id, 'update');
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        -- update reservation_changes
        INSERT INTO rsvt.reservation_changes (reservation_id, op) VALUES (OLD.id, 'delete');
    END IF;
    -- notify a channel called reservation_update
    NOTIFY reservation_update;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservations_trigger
    AFTER INSERT OR UPDATE OR DELETE ON rsvt.reservations
    FOR EACH ROW EXECUTE PROCEDURE rsvt.reservations_trigger();
