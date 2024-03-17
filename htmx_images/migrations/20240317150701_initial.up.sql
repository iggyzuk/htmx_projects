CREATE TABLE image (
    id bigserial primary key,
    file_name VARCHAR(255),
    mime_type VARCHAR(100),
    image_data BYTEA,
    created_at TIMESTAMPTZ NOT NULL default now()
);

-- Insert:
-- INSERT INTO images (file_name, mime_type, image_data)
-- VALUES ('example.jpg', 'image/jpeg', decode('base64_encoded_image_data', 'base64'));

-- Retrieve:
-- SELECT encode(image_data, 'base64') AS base64_image FROM images WHERE id = 1;