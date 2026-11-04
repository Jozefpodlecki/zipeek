CREATE TABLE words (
    id              INTEGER PRIMARY KEY,
    simplified      VARCHAR NOT NULL,
    traditional     VARCHAR,
    pinyin          VARCHAR NOT NULL,
    pinyin_numeric  VARCHAR,
    english         VARCHAR,
    part_of_speech  VARCHAR,
    hsk2_level      INTEGER,
    hsk3_level      INTEGER,
    frequency       INTEGER,
    CONSTRAINT uq_word UNIQUE (simplified, pinyin)
);

CREATE INDEX idx_words_hsk3 ON words (hsk3_level);
CREATE INDEX idx_words_hsk2 ON words (hsk2_level);

CREATE TABLE word_hsk_membership (
    word_id INTEGER NOT NULL REFERENCES words(id),
    system VARCHAR NOT NULL CHECK (system IN ('hsk2', 'hsk3')),
    level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 7),
    PRIMARY KEY (word_id, system)
);

CREATE TABLE word_pos_membership (
    word_id INTEGER NOT NULL REFERENCES words(id),
    pos VARCHAR NOT NULL,
    PRIMARY KEY (word_id, pos)
);

CREATE TABLE word_senses (
    word_id INTEGER NOT NULL REFERENCES words(id),
    sense_order INTEGER NOT NULL CHECK (sense_order > 0),
    gloss VARCHAR NOT NULL CHECK (length(trim(gloss)) > 0),
    source VARCHAR NOT NULL CHECK (source IN ('hsk-primary', 'cc-cedict-fallback')),
    source_ref INTEGER NOT NULL CHECK (source_ref > 0),
    source_gloss_order INTEGER NOT NULL CHECK (source_gloss_order > 0),
    PRIMARY KEY (word_id, sense_order)
);

CREATE TABLE sense_display (
    word_id INTEGER NOT NULL,
    sense_order INTEGER NOT NULL,
    definition VARCHAR CHECK (definition IS NULL OR length(trim(definition)) > 0),
    derivation VARCHAR NOT NULL CHECK (derivation IN ('source_verbatim', 'explicit_annotation_removed', 'relation_only')),
    CHECK (derivation = 'relation_only' AND definition IS NULL OR derivation <> 'relation_only' AND definition IS NOT NULL),
    PRIMARY KEY (word_id, sense_order),
    FOREIGN KEY (word_id, sense_order) REFERENCES word_senses(word_id, sense_order)
);

CREATE TABLE sense_relations (
    word_id INTEGER NOT NULL,
    sense_order INTEGER NOT NULL,
    relation_order INTEGER NOT NULL CHECK (relation_order > 0),
    relation_type VARCHAR NOT NULL CHECK (relation_type IN ('abbreviation_of', 'short_for', 'variant_of', 'also_written_as', 'see_also')),
    target_simplified VARCHAR NOT NULL CHECK (length(trim(target_simplified)) > 0),
    target_traditional VARCHAR,
    target_pinyin VARCHAR,
    source_annotation VARCHAR NOT NULL CHECK (length(trim(source_annotation)) > 0),
    extraction_method VARCHAR NOT NULL CHECK (extraction_method IN ('explicit_source_annotation')),
    PRIMARY KEY (word_id, sense_order, relation_order),
    FOREIGN KEY (word_id, sense_order) REFERENCES word_senses(word_id, sense_order)
);

CREATE TABLE parts_of_speech (
    code VARCHAR PRIMARY KEY,
    label VARCHAR NOT NULL
);
CREATE TABLE categories (
    slug VARCHAR PRIMARY KEY,
    label VARCHAR NOT NULL,
    taxonomy_version INTEGER NOT NULL CHECK (taxonomy_version > 0)
);
CREATE TABLE classification_runs (
    id BIGINT PRIMARY KEY,
    model VARCHAR NOT NULL,
    prompt_version VARCHAR NOT NULL,
    taxonomy_version INTEGER NOT NULL CHECK (taxonomy_version > 0),
    input_digest VARCHAR NOT NULL
);
CREATE TABLE sense_pos (
    word_id INTEGER NOT NULL,
    sense_order INTEGER NOT NULL,
    pos_code VARCHAR NOT NULL REFERENCES parts_of_speech(code),
    classification_run_id BIGINT NOT NULL REFERENCES classification_runs(id),
    PRIMARY KEY (word_id, sense_order, pos_code),
    FOREIGN KEY (word_id, sense_order) REFERENCES word_senses(word_id, sense_order)
);
CREATE TABLE sense_categories (
    word_id INTEGER NOT NULL,
    sense_order INTEGER NOT NULL,
    category_slug VARCHAR NOT NULL REFERENCES categories(slug),
    classification_run_id BIGINT NOT NULL REFERENCES classification_runs(id),
    PRIMARY KEY (word_id, sense_order, category_slug),
    FOREIGN KEY (word_id, sense_order) REFERENCES word_senses(word_id, sense_order)
);

INSERT INTO parts_of_speech (code, label) VALUES ('adjective','Adjective'),('adjective_adverbial','AdjectiveAdverbial'),('adjective_morpheme','AdjectiveMorpheme'),('adjective_nominal','AdjectiveNominal'),('non_predicate_adjective','NonPredicateAdjective'),('conjunction','Conjunction'),('adverb','Adverb'),('adverb_morpheme','AdverbMorpheme'),('interjection','Interjection'),('directional_locality','DirectionalLocality'),('morpheme','Morpheme'),('prefix','Prefix'),('idiom','Idiom'),('abbreviation','Abbreviation'),('suffix','Suffix'),('fixed_expression','FixedExpression'),('numeral','Numeral'),('numeric_morpheme','NumericMorpheme'),('noun','Noun'),('noun_morpheme','NounMorpheme'),('personal_name','PersonalName'),('place_name','PlaceName'),('organization_name','OrganizationName'),('nominal_string','NominalString'),('other_proper_noun','OtherProperNoun'),('onomatopoeia','Onomatopoeia'),('preposition','Preposition'),('classifier','Classifier'),('pronoun','Pronoun'),('pronoun_morpheme','PronounMorpheme'),('space_word','SpaceWord'),('time_word','TimeWord'),('time_word_morpheme','TimeWordMorpheme'),('auxiliary','Auxiliary'),('verb','Verb'),('verb_adverbial','VerbAdverbial'),('verb_morpheme','VerbMorpheme'),('verb_nominal','VerbNominal'),('symbol_punctuation','SymbolPunctuation'),('unclassified','Unclassified'),('modal_particle','ModalParticle'),('descriptive','Descriptive');
INSERT INTO categories (slug, label, taxonomy_version) VALUES ('travel','Travel',1),('transport','Transport',1),('food','Food and drink',1),('shopping','Shopping',1),('family','Family',1),('education','Education',1),('work','Work',1),('health','Health',1),('nature','Nature',1),('time','Time',1),('home','Home',1),('communication','Communication',1);
