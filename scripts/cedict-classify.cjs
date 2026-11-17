#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const ROOT = path.resolve(__dirname, '..');
const SOURCE = path.join(ROOT, 'crates/zipeek-db-gen/data/cc-cedict-2026-10-26.u8');
const OUTPUT = path.join(ROOT, 'crates/zipeek-db-gen/data/classifications/cc-cedict-2026-10-26-classified.json');
const SUMMARY = path.join(ROOT, 'crates/zipeek-db-gen/data/classifications/README.md');
const CATEGORY_PATTERNS = {
  travel: [/\b(?:tourist|tourism|sightseeing|vacation|holiday trip|passport|visa|itinerary|pilgrim|backpacking)\b/i],
  transport: [/\b(?:railway|railroad|train|bus stop|subway|metro station|vehicle|traffic|taxi|airplane|aircraft|airport|tram|ferry|transportation|bus ticket|train ticket|ticket for (?:a )?(?:train|bus|flight))\b/i],
  food: [/\b(?:fruit|vegetable|edible|food|snack|meal|beverage|drink|meat|fish dish|rice dish|noodle|dumpling|bread|cake|cheese|dessert|apple|banana|orange fruit|tea leaves|coffee beans)\b/i],
  cooking: [/\b(?:recipe|to cook|to bake|to fry|to roast|to steam (?:food|vegetables|fish)|stir.fry|culinary|food preparation|marinade|cooking pot|cooking method)\b/i],
  restaurants: [/\b(?:restaurant|dining room|waiter|waitress|menu|catering|cafeteria|eatery)\b/i],
  shopping: [/\b(?:to shop|shopping|retail|supermarket|shopping mall|shopkeeper|storefront|to purchase|customer in a shop)\b/i],
  money: [/\b(?:currency|banknote|coinage|cash|payment|money|yuan|dollar|pound sterling|euro|to pay|to owe money|price tag|debt)\b/i],
  finance: [/\b(?:banking|bank account|bank loan|finance|financial|stock market|securities|investment|investor|interest rate|insurance|accounting|mortgage|credit card)\b/i],
  business: [/\b(?:commercial|commerce|business|entrepreneur|corporation|company|trade agreement|merchandise|marketing|wholesale)\b/i],
  work: [/\b(?:employee|employer|occupation|profession|salary|payroll|workplace|unemployment|to resign|to hire|career|job interview)\b/i],
  industry: [/\b(?:factory|manufactur|industrial|assembly line|refinery|production plant|machine tool)\b/i],
  agriculture: [/\b(?:agricultur|farming|farmer|livestock|plantation|harvest|to cultivate crops|arable land|paddy field)\b/i],
  family: [/\b(?:mother|father|daughter|son|brother|sister|grandparent|uncle|aunt|nephew|niece|spouse|husband|wife|parent|family member|kinship|ancestor|relative)\b/i],
  relationships: [/\b(?:friendship|boyfriend|girlfriend|dating|romantic relationship|fianc[eé]|social relationship)\b/i],
  education: [/\b(?:school|student|teacher|homework|exam(?:ination)?|university|college|lecture|classroom|curriculum|to study for|education|pupil|kindergarten)\b/i],
  language: [/\b(?:grammar|linguistic|vocabulary|pronoun|adverb|adjective|noun|verb|classifier word|translation|to translate|dialect|phonetic|syllable|pinyin|language)\b/i],
  literature: [/\b(?:novel|poetry|poem|poet|literary|manuscript|fiction|prose|author|short story)\b/i],
  arts: [/\b(?:painting|sculpture|artist|calligraphy|artwork|fine arts|exhibition|drawing in art)\b/i],
  music: [/\b(?:musical|music|melody|composer|singer|orchestra|song|piano|violin|guitar|concert|singing)\b/i],
  film_media: [/\b(?:film|movie|cinema|television|tv series|newspaper|journalist|broadcast|radio station|news report|documentary)\b/i],
  entertainment: [/\b(?:entertainment|amusement|circus|leisure activity|recreation|performance for fun)\b/i],
  sports: [/\b(?:football|soccer|basketball|baseball|tennis|athlete|athletics|sports? competition|olympic|marathon|gymnastics|swimming race|stadium)\b/i],
  games: [/\b(?:board game|video game|computer game|chess|mahjong|poker|puzzle game|gameplay|gaming)\b/i],
  health: [/\b(?:wellness|healthy|health|illness|injury|symptom|fever|fatigue|headache|disability|sickness|painful)\b/i],
  medicine: [/\b(?:medical|medicine|doctor|nurse|hospital|clinic|surgery|treatment|diagnos|drug for|pharmaceut|disease|syndrome|infection|vaccine|pathology|anatomical disorder|antibiotic)\b/i],
  body: [/\b(?:body part|human body|heart muscle|kidney|liver|lung|stomach|skull|brain stem|bone|skeleton|eye socket|fingernail|tooth|muscle|vein|artery)\b/i],
  psychology: [/\b(?:psycholog|cognition|cognitive|memory|consciousness|mental process|perception|subconscious|personality disorder)\b/i],
  emotions: [/\b(?:emotion|angry|anger|sadness|grief|joyful|happiness|anxiety|afraid|fear|jealousy|disgust|loneliness|to love|to hate)\b/i],
  nature: [/\b(?:ecosystem|wilderness|natural habitat|wildlife|nature reserve|biodiversity)\b/i],
  animals: [/\b(?:animal|mammal|reptile|amphibian|insect|bird|fish species|horse|cow|pig|sheep|dog|cat|elephant|tiger|lion|rabbit|bear|monkey|butterfly|mosquito)\b/i],
  plants: [/\b(?:plant species|botan|tree|flower|grass|shrub|leaf|leaves|root of a plant|bamboo|orchid|rose flower|lotus)\b/i],
  weather: [/\b(?:weather|climate|typhoon|hurricane|tornado|thunder|lightning|rainfall|snowstorm|hailstorm|monsoon|drizzle|frost|humidity)\b/i],
  geography: [/\b(?:mountain|river|lake|ocean|sea coast|island|peninsula|valley|desert|plateau|geograph|continent|province of|city in|county in|latitude|longitude)\b/i],
  environment: [/\b(?:pollution|recycl|conservation|sustainab|carbon emission|environmental protection|greenhouse gas|ecological damage)\b/i],
  time: [/\b(?:yesterday|tomorrow|today|calendar|century|decade|hour of the day|minute of time|second of time|midnight|noon|weekday|season of the year)\b/i],
  space: [/\b(?:spatial|dimension in space|geometric space|spatial position|northward|southward|eastward|westward)\b/i],
  home: [/\b(?:household|home furnishings|domestic household|furniture|sofa|bedroom|kitchen appliance|living room|toilet bowl)\b/i],
  buildings: [/\b(?:building|architecture|architect|bridge structure|construction site|skyscraper|roof|wall of a building|building materials|temple building)\b/i],
  clothing: [/\b(?:clothing|clothes|garment|dress|trousers|pants|shirt|jacket|coat|socks|shoes|hat|sweater|skirt|uniform|fabric worn)\b/i],
  personal_care: [/\b(?:cosmetic|makeup|shampoo|soap|perfume|toothbrush|toothpaste|haircut|skin care|shaving|nail polish)\b/i],
  communication: [/\b(?:telephone|to phone|phone call|message sent|to communicate|communication|letter to someone|correspondence|postcard|to text someone)\b/i],
  technology: [/\b(?:electronics|electronic device|robotics|gadget|technolog|semiconductor|electrical engineering|battery pack|circuit board|3d printer)\b/i],
  computing: [/\b(?:computer|software|programming|algorithm|database|operating system|processor|cpu|keyboard|computer mouse|memory chip|binary code|programmer)\b/i],
  internet: [/\b(?:internet|website|webpage|online|email|social media|web browser|wi-fi|wifi|network protocol|server hosting)\b/i],
  science: [/\b(?:scientific research|scientist|science laboratory|experiment in science)\b/i],
  mathematics: [/\b(?:mathematics|mathematical|algebra|calculus|geometry|equation|integer|fraction|arithmetic|logarithm|trigonometry|statistic(?:s|al))\b/i],
  physics: [/\b(?:physics|quantum|particle physic|electric field|magnetic field|gravitational|thermodynamic|newtonian)\b/i],
  chemistry: [/\b(?:chemistry|chemical|molecule|atom|acid|alkaline|oxidation|catalyst|chemical element|compound in chemistry)\b/i],
  biology: [/\b(?:biology|biological|cell division|genetic|dna|rna|evolutionary|microorganism|chromosome)\b/i],
  law: [/\b(?:legal|lawyer|court of law|lawsuit|laws of|judicial|constitution|penal|to sue|defendant|plaintiff|regulation under law)\b/i],
  government: [/\b(?:government|governmental|ministry of|state council|mayor|governor|bureaucrat|civil servant|municipal administration)\b/i],
  politics: [/\b(?:political|politics|election|politician|parliament|legislature|democracy|communism|political party)\b/i],
  military: [/\b(?:military|army|navy|air force|soldier|troops|battlefield|weaponry|warship|battalion|general in army)\b/i],
  religion: [/\b(?:religion|religious|buddhist|buddhism|taoist|daoist|christian|christianity|muslim|islamic|mosque|monastery|scripture|worship|prayer)\b/i],
  philosophy: [/\b(?:philosophy|philosophical|metaphysics|epistemology|ethics in philosophy|ontology)\b/i],
  history: [/\b(?:history|historical|dynasty|ancient era|imperial period|archaeolog|historian|medieval)\b/i],
  culture: [/\b(?:cultural|culture|tradition|traditional custom|folklore|festival|ritual|heritage|ethnic custom)\b/i],
  society: [/\b(?:social class|social status|society|sociolog|community organization|public welfare|population census|demographic)\b/i],
  safety: [/\b(?:safety|emergency|firefighting|firefighter|to rescue|rescue worker|hazard|dangerous substance|security guard|alarm system)\b/i],
  measurement: [/\b(?:unit of measure|unit of length|unit of weight|measuring instrument|centimeter|kilometer|kilogram|meter of length|litre|liter of volume|temperature measurement)\b/i],
  colors: [/\b(?:color|colour|shade of red|shade of blue|hue|pigment)\b/i],
  materials: [/\b(?:material|metal|steel|iron|copper|aluminum|aluminium|plastic|glassware|ceramic|porcelain|wooden|timber|rubber|leather|cotton fiber|silk fabric)\b/i],
};
const CATEGORIES = Object.keys(CATEGORY_PATTERNS);
const POS = new Set(['adjective','adjective_adverbial','adjective_morpheme','adjective_nominal','non_predicate_adjective','conjunction','adverb','adverb_morpheme','interjection','directional_locality','morpheme','prefix','idiom','abbreviation','suffix','fixed_expression','numeral','numeric_morpheme','noun','noun_morpheme','personal_name','place_name','organization_name','nominal_string','other_proper_noun','onomatopoeia','preposition','classifier','pronoun','pronoun_morpheme','space_word','time_word','time_word_morpheme','auxiliary','verb','verb_adverbial','verb_morpheme','verb_nominal','symbol_punctuation','unclassified','modal_particle','descriptive']);

function classify(gloss) {
  let text = gloss.trim();
  let pos = 'unclassified';
  let grammaticalEvidence = null;
  if (/^(?:variant of|see also|see |old variant of|archaic variant of|same as|alternative form of|abbr\. for|abbr\. of)/i.test(text)) {
    return {pos: ['unclassified'], categories: [], method: 'cross_reference', review: true, evidence: []};
  }
  const posRules = [
    ['personal_name', /\b(?:surname|given name|personal name|family name)\b/i],
    ['place_name', /\b(?:place name|name of (?:a )?(?:town|city|county|province|river|mountain)|(?:city|county|province|town|village) in (?:China|Taiwan|Japan|Korea|India|France|Germany|Italy|Russia|the US|the UK))\b/i],
    ['organization_name', /\b(?:name of (?:a )?(?:company|organization|university)|(?:company|organization) name)\b/i],
    ['classifier', /^(?:\(?(?:classifier|measure word)\)?\s+(?:for|used))/i],
    ['idiom', /^\(?idiom\)?\b/i],
    ['onomatopoeia', /\b(?:onomatopoeia|onomatopoeic)\b/i],
    ['interjection', /^\(?interjection\)?\b/i],
    ['pronoun', /^\(?pronoun\)?\b/i],
    ['preposition', /^\(?preposition\)?\b/i],
    ['conjunction', /^\(?conjunction\)?\b/i],
    ['adverb', /^\(?(?:adverb|adv\.)\)?\b/i],
    ['numeral', /^(?:number|numeral)\s+(?:one|two|three|four|five|six|seven|eight|nine|ten|\d+)/i],
    ['verb', /^(?:\([^)]*\)\s*)?to\s+(?:[a-z]|be\s)/i],
    ['adjective', /^\(?adjective\)?\b/i],
    ['noun', /^(?:an?|the)\s+(?:[a-z][\w-]*\s+){0,3}(?:person|object|device|tool|fruit|plant|animal|machine|vehicle|building|institution|instrument|substance|disease|food|dish|container|piece|unit)\b/i],
  ];
  for (const [value, re] of posRules) if (re.test(text)) {pos = value; grammaticalEvidence = value; break;}
  const hits=[];
  for (const [cat, pats] of Object.entries(CATEGORY_PATTERNS)) {
    for (const pat of pats) {
      const match = pat.exec(text);
      if (match) {hits.push({cat,match:match[0]});break;}
    }
  }

  if (hits.some(x=>x.cat==='language') && /^(?:to\s+|\(?(?:adverb|adjective|noun|verb|pronoun)\)?$)/i.test(text)) {
    const i=hits.findIndex(x=>x.cat==='language'); if(i>=0) hits.splice(i,1);
  }

  const kept = hits.slice(0,2);
  return {pos:[pos],categories:kept.map(x=>x.cat),method:'gloss_rules_v1',review:pos==='unclassified'||hits.length>2,evidence:kept.map(x=>({category:x.cat,term:x.match})), ...(grammaticalEvidence?{pos_evidence:grammaticalEvidence}:{})};
}
function run() {
  const buf=fs.readFileSync(SOURCE);
  const sha=crypto.createHash('sha256').update(buf).digest('hex');
  const lines=buf.toString('utf8').split(/\r?\n/);
  const records=[];
  let entryCount=0, skipped=0;
  for(let i=0;i<lines.length;i++){
    const line=lines[i].trim(); if(!line||line.startsWith('#'))continue;
    const m=/^(\S+)\s+(\S+)\s+\[([^\]]+)\]\s+\/(.*)\/$/.exec(line);
    if(!m){skipped++; continue;}
    entryCount++;
    const glosses=m[4].split('/').map(s=>s.trim()).filter(Boolean);
    for (let j=0;j<glosses.length;j++){
      const labels=classify(glosses[j]);
      for(const code of labels.pos) if(!POS.has(code)) throw Error(`Unknown POS ${code}`);
      for(const code of labels.categories) if(!CATEGORIES.includes(code)) throw Error(`Unknown category ${code}`);
      records.push({source_line:i+1,gloss_index:j+1,traditional:m[1],simplified:m[2],pinyin:m[3],english_gloss:glosses[j],pos:labels.pos,categories:labels.categories,classification_method:labels.method,needs_review:labels.review,evidence:labels.evidence});
    }
  }
  if(skipped) throw Error(`${skipped} malformed CC-CEDICT entries`);
  fs.mkdirSync(path.dirname(OUTPUT),{recursive:true});
  fs.writeFileSync(OUTPUT,JSON.stringify(records));
  const reviewed=records.filter(x=>!x.needs_review).length;
  const classified=records.filter(x=>x.pos[0]!=='unclassified'||x.categories.length).length;
  const withPos=records.filter(x=>x.pos[0]!=='unclassified').length;
  const withCategory=records.filter(x=>x.categories.length).length;
  const manifest={source:'CC-CEDICT',source_file:path.basename(SOURCE),source_sha256:sha,export_sha256:hash(fs.readFileSync(OUTPUT)),license:'CC BY-SA 4.0',taxonomy_version:2,classification_method:'deterministic_gloss_rules_v1; not AI-reviewed',entries:entryCount,senses:records.length,nonempty_labels:classified,needs_review:records.length-reviewed};
  fs.writeFileSync(path.join(path.dirname(OUTPUT),'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
  const summary=`# CC-CEDICT gloss-classification export\n\nSource: CC-CEDICT 2026-10-26 (CC BY-SA 4.0), SHA-256 \`${sha}\`.\n\n- Dictionary entries: ${entryCount.toLocaleString()}\n- Individual gloss records: ${records.length.toLocaleString()}\n- Records with at least one nonempty automatic label: ${classified.toLocaleString()}\n- Records with grammatical label other than unclassified: ${withPos.toLocaleString()}\n- Records with topical category: ${withCategory.toLocaleString()}\n- Records flagged for review: ${(records.length-reviewed).toLocaleString()}\n\n**Important**: These are deterministic pattern-based suggestions, **not classifications manually or individually reviewed by ChatGPT**. Every gloss is present, with explicit unclassified results where evidence is insufficient. A generic subject-word match may still yield a false positive; inspect evidence and review before database publication. The original source-line number and gloss index are stable locators within the source file; they are **not** HSK database word/sense IDs. The production db-gen classification cache has a different schema and source-sense digest, so this export must not be pasted into that cache.\n\nRegenerate from repository root with \`node scripts/cedict-classify.cjs\`.\n`;
  fs.writeFileSync(SUMMARY,summary);
  console.log(JSON.stringify({entries:entryCount,senses:records.length,classified,withPos,withCategory,needs_review:records.length-reviewed,sha256:sha,bytes:fs.statSync(OUTPUT).size},null,2));
}
if(require.main===module)run();
module.exports={classify,run};
