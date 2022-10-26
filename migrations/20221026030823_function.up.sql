-- if both set, find all reservations within during for the resource and user
CREATE OR REPLACE FUNCTION rsvt.query(uid text, rid text, during TSTZRANGE) RETURNS TABLE (LIKE rsvt.reservations) AS $$
    BEGIN
        IF uid IS NULL AND rid IS NULL THEN
        -- if both are null, find all reservations within during
            RETURN QUERY SELECT * FROM rsvt.reservations WHERE during && rsvt.reservations.rperiod;
        ELSIF uid IS NULL AND rid IS NOT NULL THEN
        -- if user_id is null, find all reservations within during for the resource
            RETURN QUERY SELECT * FROM rsvt.reservations WHERE resource_id = rid AND during @> rsvt.reservations.during;
        ELSIF uid IS NOT NULL AND rid IS NULL THEN
        -- if resource_id is null, find all reservations within during for the user
            RETURN QUERY SELECT * FROM rsvt.reservations WHERE user_id = uid AND during @> rsvt.reservations.during;
        ELSE
        -- if both set, find all reservations within during for the resource and user
            RETURN QUERY SELECT * FROM rsvt.reservations WHERE user_id = uid AND resource_id = rid AND during @> rsvt.reservations.during;
        END IF;
    END;
$$ LANGUAGE plpgsql;
