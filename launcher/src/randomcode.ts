import { randomInt } from "crypto";

const CHOICES: {
  [lang: string]: { starts: string[]; middles: string[]; ends: string[] };
} = {
  en: {
    starts: [
      "adjective",
      "agameof",
      "alpha",
      "anti",
      "availableto",
      "awfullyhot",
      "awhisperof",
      "bad",
      "balanced",
      "beholdthe",
      "better",
      "big",
      "bingussed",
      "blind",
      "boneless",
      "boosted",
      "bottom",
      "bubbly",
      "bugged",
      "central",
      "chilly",
      "chonky",
      "cobbing",
      "code",
      "cold",
      "compiled",
      "cool",
      "corny",
      "cursed",
      "dancing",
      "dang",
      "dark",
      "deleted",
      "electric",
      "endless",
      "epic",
      "error",
      "famous",
      "feedthe",
      "fistfullof",
      "flying",
      "free",
      "ghostly",
      "giga",
      "goodluck",
      "harbingerof",
      "heavy",
      "hello",
      "helpme",
      "herecomes",
      "herecomes",
      "higsbys",
      "hot",
      "hyper",
      "im",
      "impossible",
      "infamous",
      "intense",
      "invisible",
      "itsme",
      "jackin",
      "lacking",
      "legsgo",
      "licenseto",
      "lookup",
      "lucky",
      "lv100",
      "mega",
      "moon",
      "murky",
      "nebula",
      "net",
      "new",
      "nice",
      "nicebig",
      "no",
      "nonstop",
      "official",
      "old",
      "one",
      "online",
      "open",
      "overwhelming",
      "player",
      "pocketfullof",
      "poggy",
      "popped",
      "protect",
      "rad",
      "relaxed",
      "returnof",
      "revengeofthe",
      "risen",
      "risky",
      "running",
      "scary",
      "shining",
      "shrimpy",
      "spectacular",
      "spiffy",
      "spooky",
      "starforce",
      "stolen",
      "sus",
      "sword",
      "team",
      "techno",
      "the",
      "thefinal",
      "thesearchfor",
      "thicc",
      "thick",
      "throwthe",
      "top",
      "totallyaccurate",
      "unhinged",
      "unlikely",
      "uwu",
      "verbing",
      "veteran",
      "vibing",
      "weird",
      "whohere",
      "winking",
      "yoinky",
    ],

    middles: [
      "airhockey",
      "airraid",
      "airshot",
      "airspin",
      "alpaca",
      "alpha",
      "antidmg",
      "antinavi",
      "antirecov",
      "antisword",
      "anubis",
      "apple",
      "aquadrgn",
      "aquasword",
      "areagrab",
      "assnsword",
      "aurahead",
      "baby",
      "balance",
      "balanced",
      "bambsword",
      "barrier",
      "bass",
      "bassanly",
      "bcc",
      "bdt",
      "bigbomb",
      "bighook",
      "billy",
      "bingus",
      "blackbomb",
      "blast",
      "blues",
      "bogos",
      "bomboy",
      "bombcorn",
      "boomer",
      "boyfriends",
      "browser",
      "brs",
      "bubble",
      "bubbleman",
      "bugbomb",
      "bugfix",
      "bugfrag",
      "bunny",
      "burrito",
      "busterup",
      "cannon",
      "canodumb",
      "cat",
      "charge",
      "chicken",
      "chiptrader",
      "chonk",
      "circlegun",
      "circles",
      "circus",
      "cob",
      "coffeepot",
      "colarmy",
      "coldbear",
      "coldman",
      "colonel",
      "colorpoint",
      "command",
      "content",
      "cornshot",
      "cowboy",
      "crackshot",
      "crossdivide",
      "cucumber",
      "damage",
      "damnswrd",
      "daniel",
      "digeridoo",
      "discord",
      "dive",
      "dog",
      "dollthunder",
      "donut",
      "doubleshot",
      "drcossak",
      "drillarm",
      "duo",
      "dust",
      "eguchi",
      "eleball",
      "elec",
      "elecdrgb",
      "elecpulse",
      "elecsword",
      "element",
      "elemtrap",
      "energybomb",
      "erase",
      "error",
      "falzar",
      "fan",
      "fanfare",
      "fastgauge",
      "firebrn",
      "firehit",
      "firesword",
      "fish",
      "fishanly",
      "flashbomb",
      "friday",
      "fullcust",
      "geddon",
      "golemhit",
      "grabbanish",
      "gregar",
      "ground",
      "guardian",
      "gundels",
      "havefun",
      "heat",
      "hiboomer",
      "holypanel",
      "hub",
      "hubbatch",
      "humor",
      "ice",
      "iceball",
      "iceseed",
      "imfish",
      "iminthecode",
      "invisible",
      "ironshell",
      "judge",
      "justiceone",
      "lan",
      "lance",
      "landing",
      "lifeaura",
      "lifesync",
      "lilboiler",
      "machgun",
      "magcoil",
      "magnum",
      "man",
      "meaman",
      "megaman",
      "melody",
      "meteorknuckle",
      "meteors",
      "mettaur",
      "mine",
      "minibomb",
      "monday",
      "moloko",
      "moon",
      "moonblade",
      "mrfamous",
      "muramasa",
      "navi",
      "navicust",
      "needle",
      "neovariable",
      "netbattle",
      "netbattler",
      "nightmare",
      "noun",
      "one",
      "operator",
      "pengi",
      "permahole",
      "pet",
      "pirate",
      "poutine",
      "prinkus",
      "proto",
      "qbot",
      "qforce",
      "quaker",
      "rabbit",
      "recovery",
      "reflector",
      "riskyhoney",
      "rock",
      "rockcube",
      "roll",
      "rollinglog",
      "routine",
      "rush",
      "sanctuary",
      "sand",
      "sandworm",
      "sensor",
      "sequel",
      "shark",
      "shield",
      "shrubby",
      "silence",
      "skill",
      "slash",
      "slowgauge",
      "snake",
      "spout",
      "spreadr",
      "starfish",
      "static",
      "stepsword",
      "stevejobs",
      "suprvulc",
      "tango",
      "tankcan",
      "tengu",
      "thawk",
      "thegiantfist",
      "thunder",
      "timpani",
      "toiletmet",
      "tomahawk",
      "tornado",
      "trainarrow",
      "tripleshot",
      "uninstall",
      "uwu",
      "vdoll",
      "vulcan",
      "wavearm",
      "whitecap",
      "widesht",
      "widesword",
      "win",
      "wind",
      "windrack",
      "wood",
      "wooddrgn",
      "www",
      "yoyo",
      "zenny",
      "zero",
    ],

    ends: [
      "6",
      "aaaaaa",
      "alpha",
      "amogus",
      "angy",
      "area",
      "aura",
      "banned",
      "battle",
      "beastmode",
      "bimbus",
      "bingus",
      "binted",
      "blessing",
      "blubblub",
      "bot",
      "burrito",
      "chip",
      "chonked",
      "clowntown",
      "cob",
      "cobbers",
      "combo",
      "congratulations",
      "cornfusion",
      "cornout",
      "crasher",
      "damn",
      "data",
      "denizen",
      "eguchiwut",
      "endofstring",
      "energy",
      "error",
      "exe",
      "execute",
      "experience",
      "extra",
      "faked",
      "fartspin",
      "forme",
      "fortnite",
      "frenzy",
      "gauntlet",
      "ggswp",
      "grandprix",
      "greatplay",
      "hamachi",
      "heehoo",
      "helpimtrappedinhere",
      "hour",
      "huh",
      "hype",
      "impression",
      "isa",
      "isbalanced",
      "issue",
      "iswinning",
      "jello",
      "legabed",
      "letmeout",
      "license",
      "lilguy",
      "loicense",
      "longsword",
      "lovemegaman",
      "man",
      "megalegs",
      "meme",
      "minna",
      "mojo",
      "occurroico",
      "omega",
      "parttwo",
      "party",
      "pause",
      "power",
      "powerhour",
      "progchamp",
      "programadvance",
      "ratioed",
      "reg",
      "rollback",
      "rotango",
      "sfboy",
      "shmooving",
      "sickos",
      "slimetier",
      "snapped",
      "solution",
      "sp",
      "spam",
      "sploinky",
      "sprite",
      "stevejobs",
      "strategy",
      "swag",
      "swaggums",
      "swarm",
      "symeseus",
      "tag",
      "technology",
      "tfc",
      "thunder",
      "tier",
      "tiltcontrols",
      "time",
      "toptier",
      "ultrafiesta",
      "unchained",
      "uninstalled",
      "unlegs",
      "uwu",
      "vbalink",
      "victor",
      "wavedash",
      "wswalk",
      "x2",
      "yeastmode",
      "yeet",
      "youareworthy",
      "yourewinner",
      "yum",
    ],
  },

  ja: {
    starts: [
      "abunai",
      "anoo",
      "bokuno",
      "dareka",
      "haroharo",
      "hazimete",
      "hontouno",
      "ii",
      "intaanetto",
      "itiban",
      "mada",
      "maji",
      "makenai",
      "masaka",
      "meta",
      "mettya",
      "metyakuyta",
      "minna",
      "muzukasii",
      "mou",
      "naisu",
      "nee",
      "ohayou",
      "oreno",
      "sasuga",
      "sausage",
      "singuru",
      "sugee",
      "sugoi",
      "sugu",
      "syainii",
      "tasikani",
      "tokorode",
      "toriaezu",
      "toripuru",
      "tuyoi",
      "tuyosugiru",
      "wagahaino",
      "yoi",
      "yorosiku",
      "zannen",
      "zenzen",
      "zibunno",
      "zituwa",
    ],
    middles: [
      "arigatou",
      "basutaa",
      "batoru",
      "dekao",
      "dhuuo",
      "eguchi",
      "faruzaa",
      "furusinkuro",
      "enzan",
      "hontounotikara",
      "geemu",
      "guranpuri",
      "gureiga",
      "iiwake",
      "ikkai",
      "kasutamu",
      "kimoti",
      "mainkurafuto",
      "mangoo",
      "masutazukurasu",
      "meizin",
      "mondai",
      "netto",
      "nige",
      "nontan",
      "onegai",
      "rokkuman",
      "siniakurasu",
      "susi",
      "sutegapanesuti",
      "syoubu",
      "taisen",
      "tango",
      "tikara",
      "tisao",
    ],
    ends: [
      "bakari",
      "daizyoubu",
      "dake",
      "desu",
      "faito",
      "gozaimasu",
      "hazimemasite",
      "hazu",
      "ikanai",
      "ikenai",
      "itadakimasu",
      "kokonattu",
      "kudasai",
      "kure",
      "kusa",
      "makenai",
      "maketa",
      "miseteyaro",
      "mittetekure",
      "nai",
      "onegai",
      "sikatanai",
      "simasu",
      "simasyou",
      "sitteruno",
      "sugee",
      "sugiru",
      "sukanai",
      "syouganai",
      "taisen",
      "tasukete",
      "tenkyuu",
      "tigau",
      "uwaa",
      "wakaranai",
      "warota",
      "yabe",
      "yaritai",
      "yaritakunai",
      "yaru",
      "yatta",
      "ze",
      "zyanai",
    ],
  },
};

export default function randomCode(lang: string) {
  if (!Object.prototype.hasOwnProperty.call(CHOICES, lang)) {
    lang = "en";
  }
  const choices = CHOICES[lang];
  return `${choices.starts[randomInt(0, choices.starts.length)]}-${
    choices.middles[randomInt(0, choices.middles.length)]
  }-${choices.ends[randomInt(0, choices.ends.length)]}`.slice(0, 40);
}
