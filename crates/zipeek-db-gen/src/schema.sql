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

CREATE TABLE dictionary_versions (
    snapshot_fingerprint VARCHAR PRIMARY KEY,
    provenance VARCHAR NOT NULL,
    word_count INTEGER NOT NULL CHECK (word_count >= 0),
    sense_count INTEGER NOT NULL CHECK (sense_count >= 0)
);

CREATE TABLE vocabulary_identities (
    word_id INTEGER NOT NULL REFERENCES words(id),
    reading_key VARCHAR NOT NULL,
    candidate_key VARCHAR NOT NULL,
    identity_status VARCHAR NOT NULL CHECK (identity_status IN ('unresolved', 'verified')),
    PRIMARY KEY (word_id),
    UNIQUE (candidate_key)
);

CREATE TABLE characters (
    character VARCHAR PRIMARY KEY,
    unicode_scalar INTEGER NOT NULL UNIQUE CHECK (unicode_scalar > 0)
);

CREATE TABLE character_readings (
    character VARCHAR NOT NULL,
    traditional VARCHAR NOT NULL,
    pinyin_numeric VARCHAR NOT NULL,
    source_line INTEGER NOT NULL CHECK (source_line > 0),
    PRIMARY KEY (character, pinyin_numeric)
);

CREATE TABLE word_characters (
    word_id INTEGER NOT NULL REFERENCES words(id),
    character_position INTEGER NOT NULL CHECK (character_position > 0),
    character VARCHAR NOT NULL REFERENCES characters(character),
    PRIMARY KEY (word_id, character_position)
);

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
    source VARCHAR NOT NULL CHECK (source IN ('hsk-primary', 'cc-cedict-fallback', 'cc-cedict')),
    source_ref INTEGER NOT NULL CHECK (source_ref > 0),
    source_gloss_order INTEGER NOT NULL CHECK (source_gloss_order > 0),
    PRIMARY KEY (word_id, sense_order)
);

CREATE TABLE sense_display (
    word_id INTEGER NOT NULL,
    sense_order INTEGER NOT NULL,
    definition VARCHAR CHECK (definition IS NULL OR length(trim(definition)) > 0),
    derivation VARCHAR NOT NULL CHECK (derivation IN ('source_verbatim', 'explicit_annotation_removed', 'relation_only')),
    sense_kind VARCHAR NOT NULL CHECK (sense_kind IN ('lexical', 'usage_reference', 'cross_reference')),
    CHECK (derivation = 'relation_only' AND definition IS NULL OR derivation <> 'relation_only' AND definition IS NOT NULL),
    PRIMARY KEY (word_id, sense_order),
    FOREIGN KEY (word_id, sense_order) REFERENCES word_senses(word_id, sense_order)
);

CREATE TABLE sense_qualifiers (
    word_id INTEGER NOT NULL,
    sense_order INTEGER NOT NULL,
    qualifier_kind VARCHAR NOT NULL CHECK (qualifier_kind IN ('region', 'usage', 'register')),
    qualifier_code VARCHAR NOT NULL CHECK (length(trim(qualifier_code)) > 0),
    source_annotation VARCHAR NOT NULL CHECK (length(trim(source_annotation)) > 0),
    extraction_method VARCHAR NOT NULL CHECK (extraction_method = 'explicit_source_annotation'),
    PRIMARY KEY (word_id, sense_order, qualifier_kind, qualifier_code),
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
INSERT INTO categories (slug, label, taxonomy_version) VALUES ('travel','Travel and tourism',2),('transport','Transport',2),('food','Food and drink',2),('cooking','Cooking and recipes',2),('restaurants','Restaurants and dining',2),('shopping','Shopping and retail',2),('money','Money and payments',2),('finance','Finance and banking',2),('business','Business and commerce',2),('work','Employment and work',2),('industry','Industry and manufacturing',2),('agriculture','Agriculture and farming',2),('family','Family and kinship',2),('relationships','Personal relationships',2),('education','Education and learning',2),('language','Language and linguistics',2),('literature','Literature and writing',2),('arts','Visual and performing arts',2),('music','Music and sound arts',2),('film_media','Film and media',2),('entertainment','Entertainment and leisure',2),('sports','Sports and competitions',2),('games','Games and play',2),('health','Health and well-being',2),('medicine','Medicine and clinical care',2),('body','Anatomy and physiology',2),('psychology','Psychology and cognition',2),('emotions','Emotions and feelings',2),('nature','Nature and ecosystems',2),('animals','Animals',2),('plants','Plants',2),('weather','Weather and climate',2),('geography','Geography and landforms',2),('environment','Environmental issues',2),('time','Time and chronology',2),('space','Space and spatial relations',2),('home','Home and household',2),('buildings','Buildings and architecture',2),('clothing','Clothing and fashion',2),('personal_care','Personal care and hygiene',2),('communication','Communication and messages',2),('technology','Technology and devices',2),('computing','Computing and software',2),('internet','Internet and networks',2),('science','Science and research',2),('mathematics','Mathematics',2),('physics','Physics',2),('chemistry','Chemistry',2),('biology','Biology',2),('law','Law and justice',2),('government','Government and public administration',2),('politics','Politics and ideology',2),('military','Military and defense',2),('religion','Religion and spirituality',2),('philosophy','Philosophy and ethics',2),('history','History and historical periods',2),('culture','Culture and customs',2),('society','Society and social issues',2),('safety','Safety and emergencies',2),('measurement','Measurement and quantities',2),('colors','Colors and appearance',2),('materials','Materials and substances',2);
