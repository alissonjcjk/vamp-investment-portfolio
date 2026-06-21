-- sample_assets.sql
-- Essa fixture popula o banco com 1 usuário e 3 ativos de exemplo.
-- As senhas não importam muito aqui pois lidaremos mais com a camada logada, mas o hash está mockado.

INSERT INTO users (id, username, password_hash)
VALUES (1000, 'brucewayne', '$argon2id$v=19$m=19456,t=2,p=1$u12345$a123')
ON CONFLICT DO NOTHING;

INSERT INTO assets (id, user_id, name, quantity, unit_value)
VALUES 
    (1001, 1000, 'WAYN3', 1000.0, 50.50),
    (1002, 1000, 'BTC', 1.5, 300000.00),
    (1003, 1000, 'IVVB11', 50.0, 310.20)
ON CONFLICT DO NOTHING;
