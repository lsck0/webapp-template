CREATE TABLE IF NOT EXISTS posts (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),

    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author UUID references users(id) NOT NULL,

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
