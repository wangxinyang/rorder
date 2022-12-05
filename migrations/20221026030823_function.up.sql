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
