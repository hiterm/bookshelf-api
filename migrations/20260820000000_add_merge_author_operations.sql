INSERT INTO event_operation (operation)
VALUES ('merge_as_destination')
ON CONFLICT DO NOTHING;

INSERT INTO event_set_operation (operation)
VALUES ('merge_author')
ON CONFLICT DO NOTHING;
