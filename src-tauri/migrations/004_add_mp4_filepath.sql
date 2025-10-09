-- Add mp4_filepath column to favorites table for better performance
ALTER TABLE favorites ADD COLUMN mp4_filepath TEXT;
