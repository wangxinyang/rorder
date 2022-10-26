DROP TRIGGER reservations_trigger ON rsvt.reservations;
DROP FUNCTION rsvt.reservations_trigger();
DROP TABLE rsvt.reservation_changes CASCADE;
