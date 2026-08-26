-- vari_vikiqam — tax rates, by kind, by place, by date
--
-- READ THIS BEFORE USING IT.
--
-- No rate is seeded. Not one. GST rates by HSN, TDS rates by section, income
-- tax slabs, professional tax by state, VAT where it survives — every one is
-- set by statute or notification and amended, often in a budget and sometimes
-- between them. A file written by someone who is not your tax function is not
-- where they should come from.
--
-- What is seeded is the shape, and the list of states and union territories
-- with their GST state codes. Those are identifiers rather than rates, and
-- they change rarely — but they are marked for checking too, because a wrong
-- state code puts a return in the wrong state.
--
-- Everything is effective-dated. A return for last quarter is computed on the
-- rates in force last quarter, so amending a rate means closing the old row
-- and inserting a new one, never updating in place. A filing re-run next year
-- must produce what it produced when it was filed.

CREATE TABLE IF NOT EXISTS vari_vikiqam (
    vari_vakY      TEXT    NOT NULL,  -- GST, TDS, ITR, PT, VAT, CESS …
    kuRi           TEXT    NOT NULL,  -- HSN code, TDS section, slab name, ''
    mAnilam        TEXT    NOT NULL,  -- state code, or '' for all-India
    vikiqam        REAL,              -- rate as a number out of 100
    nilYq_qokY     REAL,              -- a flat amount instead, for PT-style rules
    varampu_muqal  REAL    NOT NULL DEFAULT 0,   -- band starts at
    varampu_varY   REAL,              -- band ends at; NULL = no upper bound
    amal_qotakkam  TEXT    NOT NULL,  -- in force from, ISO date
    amal_mutivu    TEXT,              -- in force until; NULL = still current
    mUlam          TEXT    NOT NULL   -- which notification this came from
);

-- Two rules of the same kind, for the same thing, in the same place, starting
-- on the same day would make the answer depend on row order.
CREATE UNIQUE INDEX IF NOT EXISTS vari_vikiqam_orey
    ON vari_vikiqam (vari_vakY, kuRi, mAnilam, varampu_muqal, amal_qotakkam);

CREATE INDEX IF NOT EXISTS vari_vikiqam_qEti
    ON vari_vikiqam (vari_vakY, mAnilam, amal_qotakkam);

-- States and union territories, by GST state code.
CREATE TABLE IF NOT EXISTS mAnilawkaL (
    kuRi     TEXT NOT NULL,   -- GST state code
    peyar    TEXT NOT NULL,   -- name
    vakY     TEXT NOT NULL,   -- 'மாநிலம்' or 'ஒன்றியப் பகுதி'
    mUlam    TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS mAnilawkaL_kuRi ON mAnilawkaL (kuRi);

DELETE FROM mAnilawkaL WHERE mUlam LIKE 'CHECK%';

INSERT INTO mAnilawkaL (kuRi, peyar, vakY, mUlam) VALUES
    ('01', 'Jammu and Kashmir',            'ஒன்றியப் பகுதி', 'CHECK — verify against the GST portal'),
    ('02', 'Himachal Pradesh',             'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('03', 'Punjab',                       'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('04', 'Chandigarh',                   'ஒன்றியப் பகுதி', 'CHECK — verify against the GST portal'),
    ('05', 'Uttarakhand',                  'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('06', 'Haryana',                      'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('07', 'Delhi',                        'ஒன்றியப் பகுதி', 'CHECK — verify against the GST portal'),
    ('08', 'Rajasthan',                    'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('09', 'Uttar Pradesh',                'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('10', 'Bihar',                        'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('11', 'Sikkim',                       'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('12', 'Arunachal Pradesh',            'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('13', 'Nagaland',                     'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('14', 'Manipur',                      'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('15', 'Mizoram',                      'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('16', 'Tripura',                      'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('17', 'Meghalaya',                    'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('18', 'Assam',                        'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('19', 'West Bengal',                  'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('20', 'Jharkhand',                    'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('21', 'Odisha',                       'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('22', 'Chhattisgarh',                 'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('23', 'Madhya Pradesh',               'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('24', 'Gujarat',                      'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('26', 'Dadra and Nagar Haveli and Daman and Diu', 'ஒன்றியப் பகுதி', 'CHECK — verify against the GST portal'),
    ('27', 'Maharashtra',                  'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('29', 'Karnataka',                    'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('30', 'Goa',                          'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('31', 'Lakshadweep',                  'ஒன்றியப் பகுதி', 'CHECK — verify against the GST portal'),
    ('32', 'Kerala',                       'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('33', 'Tamil Nadu',                   'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('34', 'Puducherry',                   'ஒன்றியப் பகுதி', 'CHECK — verify against the GST portal'),
    ('35', 'Andaman and Nicobar Islands',  'ஒன்றியப் பகுதி', 'CHECK — verify against the GST portal'),
    ('36', 'Telangana',                    'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('37', 'Andhra Pradesh',               'மாநிலம்',        'CHECK — verify against the GST portal'),
    ('38', 'Ladakh',                       'ஒன்றியப் பகுதி', 'CHECK — verify against the GST portal');

-- Deliberately empty. Load your own, from the notification that sets them:
--
--   INSERT INTO vari_vikiqam
--       (vari_vakY, kuRi, mAnilam, vikiqam, varampu_muqal, amal_qotakkam, mUlam)
--   VALUES
--       ('GST', '8471', '', 18, 0, '2017-07-01', 'Notification 1/2017 …');
