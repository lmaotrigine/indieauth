-- Your SQL goes here

CREATE TABLE IF NOT EXISTS indieauth_codes (
  code TEXT NOT NULL UNIQUE PRIMARY KEY,
  client_id TEXT NOT NULL,
  redirect_uri TEXT NOT NULL,
  "state" TEXT NOT NULL,
  response_type TEXT NOT NULL,
  code_challenge TEXT NOT NULL,
  authorized BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS tokens (
  id TEXT NOT NULL UNIQUE PRIMARY KEY,
  sub TEXT NOT NULL,
  aud TEXT NOT NULL,
  iss TEXT NOT NULL,
  iat TEXT NOT NULL,
  exp INTEGER,
  valid INTEGER
);

CREATE TABLE IF NOT EXISTS gitlab_tokens (
  id TEXT NOT NULL UNIQUE PRIMARY KEY,
  user_id INTEGER NOT NULL,
  access_token TEXT NOT NULL,
  refresh_token TEXT NOT NULL
);
