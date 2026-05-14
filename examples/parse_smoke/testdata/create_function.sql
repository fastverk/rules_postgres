CREATE OR REPLACE FUNCTION add_one(x integer)
RETURNS integer
LANGUAGE plpgsql
AS $$
BEGIN
  RETURN x + 1;
END;
$$;
