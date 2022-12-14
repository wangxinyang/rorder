-- if both set, find all reservations within during for the resource and user
CREATE OR REPLACE FUNCTION rsvt.query(
    uid text,
    rid text,
    during TSTZRANGE,
    status rsvt.reservation_status,
    page integer default 1,
    is_desc bool default false,
    page_size integer default 10
) RETURNS TABLE (LIKE rsvt.reservations) AS $$ -- RETURNS TABLE (LIKE rsvt.reservations) 返回表
DECLARE
    _sql text;
    BEGIN
        -- if page_size is not between 10 and 100, set it to 10
        IF page_size < 10 OR page_size > 100 THEN
            page_size := 10;
        END IF;
        IF page < 1 THEN
            page := 1;
        END IF;
        -- format the qurey based on parameters
        _sql := format(
            'select * from rsvt.reservations where %L @> rperiod and rstatus = %L and %s order by lower(rperiod) %s
            limit %s offset %s',
            during,
            status,
            CASE
                WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
                WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
                WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
                ELSE 'resource_id =' || quote_literal(rid) || ' AND user_id = ' || quote_literal(uid)
            END,
            CASE
                WHEN is_desc THEN 'DESC'
                ELSE 'ASC'
            END,
            page_size,
            (page - 1) * page_size
        );

        -- log the sql
        RAISE NOTICE '%', _sql;

        -- execute the query
        RETURN QUERY EXECUTE _sql;

    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION rsvt.filter(
    uid text,
    rid text,
    status rsvt.reservation_status,
    cursor bigint default null,
    is_desc bool default false,
    page_size bigint default 10
) RETURNS TABLE (LIKE rsvt.reservations) AS $$ -- RETURNS TABLE (LIKE rsvt.reservations) 返回表
DECLARE
    _sql text;
    BEGIN
        -- if cursor is null, set it to 0 if is_desc is false, or to max int if is_desc is true
        IF cursor IS NULL or cursor < 0 THEN
            IF is_desc THEN
                cursor := 2147483647;
            ELSE
                cursor := 0;
            END IF;
        END IF;
        -- if page_size is not between 10 and 100, set it to 10
        IF page_size < 10 OR page_size > 100 THEN
            page_size := 10;
        END IF;
        -- format the qurey based on parameters
        _sql := format(
            'select * from rsvt.reservations where %s and rstatus = %L and %s order by id %s limit %L::integer',
            CASE
                WHEN is_desc THEN 'id < ' || cursor
                ELSE 'id > ' || cursor
            END,
            status,
            CASE
                WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
                WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
                WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
                ELSE 'resource_id =' || quote_literal(rid) || ' AND user_id = ' || quote_literal(uid)
            END,
            CASE
                WHEN is_desc THEN 'DESC'
                ELSE 'ASC'
            END,
            page_size
        );

        -- log the sql
        RAISE NOTICE '%', _sql;

        -- execute the query
        RETURN QUERY EXECUTE _sql;

    END;
$$ LANGUAGE plpgsql;
