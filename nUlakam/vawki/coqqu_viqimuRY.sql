-- coqqu_viqimuRY — asset classification and provisioning rules
--
-- READ THIS BEFORE USING THE ROWS BELOW.
--
-- Every figure seeded here is a PLACEHOLDER. They are shaped like real rules
-- so that the library can be tested, and they are not the rules. The day
-- counts that make an account sub-standard, and the percentages provisioned
-- against each class, are set by RBI circular and are amended; taking them
-- from a file written by someone who is not your compliance function is how a
-- return goes out wrong.
--
-- Replace them with the current circular, set mUlam to say which one, and
-- coqqu.qmz's சரிபார்க்கப்படாதவை will stop naming them.
--
-- The table is effective-dated on purpose. A review is done against the rules
-- in force on the review date, not the rules in force today, so amending a
-- rule means closing the old row and inserting a new one — never updating in
-- place. A year-end review re-run in April must produce what it produced then.

CREATE TABLE IF NOT EXISTS coqqu_viqimuRY (
    vakY              TEXT    NOT NULL,   -- the class an account falls into
    nAtkaL_muqal      INTEGER NOT NULL,   -- days overdue, from (inclusive)
    nAtkaL_varY       INTEGER,            -- days overdue, to (inclusive); NULL = no upper bound
    oqukkItu_vikiqam  REAL    NOT NULL,   -- provisioning percentage, written as 15 for 15%
    amal_qotakkam     TEXT    NOT NULL,   -- in force from, ISO date
    amal_mutivu       TEXT,               -- in force until, ISO date; NULL = still current
    mUlam             TEXT    NOT NULL    -- which circular this came from
);

-- One rule may not overlap another on the same day, or a loan would fall into
-- two classes and the answer would depend on row order.
CREATE UNIQUE INDEX IF NOT EXISTS coqqu_viqimuRY_orey
    ON coqqu_viqimuRY (nAtkaL_muqal, amal_qotakkam);

DELETE FROM coqqu_viqimuRY WHERE mUlam LIKE 'PLACEHOLDER%';

INSERT INTO coqqu_viqimuRY
    (vakY, nAtkaL_muqal, nAtkaL_varY, oqukkItu_vikiqam, amal_qotakkam, amal_mutivu, mUlam)
VALUES
    ('Standard',     0,   90,   0.40, '2000-01-01', NULL, 'PLACEHOLDER — not a real figure'),
    ('Sub-standard', 91,  455,  15.00, '2000-01-01', NULL, 'PLACEHOLDER — not a real figure'),
    ('Doubtful 1',   456, 820,  25.00, '2000-01-01', NULL, 'PLACEHOLDER — not a real figure'),
    ('Doubtful 2',   821, 1550, 40.00, '2000-01-01', NULL, 'PLACEHOLDER — not a real figure'),
    ('Doubtful 3',   1551, NULL, 100.00, '2000-01-01', NULL, 'PLACEHOLDER — not a real figure');
