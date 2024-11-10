use std::ops::Range;

pub const RIBBI_STYLE_NAMES: [&str; 5] = ["Regular", "Italic", "Bold", "BoldItalic", "Bold Italic"];
pub const STATIC_STYLE_NAMES: [&str; 18] = [
    "Thin",
    "ExtraLight",
    "Light",
    "Regular",
    "Medium",
    "SemiBold",
    "Bold",
    "ExtraBold",
    "Black",
    "Thin Italic",
    "ExtraLight Italic",
    "Light Italic",
    "Italic",
    "Medium Italic",
    "SemiBold Italic",
    "Bold Italic",
    "ExtraBold Italic",
    "Black Italic",
];

pub const VALID_SCRIPT_TAGS: [&str; 170] = [
    "DFLT", "adlm", "ahom", "hluw", "arab", "armn", "avst", "bali", "bamu", "bass", "batk", "beng",
    "bng2", "bhks", "bopo", "brah", "brai", "bugi", "buhd", "byzm", "cans", "cari", "aghb", "cakm",
    "cham", "cher", "chrs", "hani", "copt", "cprt", "cyrl", "DFLT", "dsrt", "deva", "dev2", "diak",
    "dogr", "dupl", "egyp", "elba", "elym", "ethi", "geor", "glag", "goth", "gran", "grek", "gujr",
    "gjr2", "gong", "guru", "gur2", "hang", "jamo", "rohg", "hano", "hatr", "hebr", "kana", "armi",
    "phli", "prti", "java", "kthi", "knda", "knd2", "kana", "kali", "khar", "kits", "khmr", "khoj",
    "sind", "lao ", "latn", "lepc", "limb", "lina", "linb", "lisu", "lyci", "lydi", "mahj", "maka",
    "mlym", "mlm2", "mand", "mani", "marc", "gonm", "math", "medf", "mtei", "mend", "merc", "mero",
    "plrd", "modi", "mong", "mroo", "mult", "musc", "mymr", "mym2", "nbat", "nand", "newa", "talu",
    "nko ", "nshu", "hmnp", "orya", "ory2", "ogam", "olck", "ital", "hung", "narb", "perm", "xpeo",
    "sogo", "sarb", "orkh", "osge", "osma", "hmng", "palm", "pauc", "phag", "phnx", "phlp", "rjng",
    "runr", "samr", "saur", "shrd", "shaw", "sidd", "sgnw", "sinh", "sogd", "sora", "soyo", "xsux",
    "sund", "sylo", "syrc", "tglg", "tagb", "tale", "lana", "tavt", "takr", "taml", "tml2", "tang",
    "telu", "tel2", "thaa", "thai", "tibt", "tfng", "tirh", "ugar", "vai ", "wcho", "wara", "yezi",
    "yi  ", "zanb",
];

pub const VALID_LANG_TAGS: [&str; 612] = [
    "dflt", "ABA ", "ABK ", "ACH ", "ACR ", "ADY ", "AFK ", "AFR ", "AGW ", "AIO ", "AKA ", "AKB ",
    "ALS ", "ALT ", "AMH ", "ANG ", "APPH", "ARA ", "ARG ", "ARI ", "ARK ", "ASM ", "AST ", "ATH ",
    "AVR ", "AWA ", "AYM ", "AZB ", "AZE ", "BAD ", "BAD0", "BAG ", "BAL ", "BAN ", "BAR ", "BAU ",
    "BBC ", "BBR ", "BCH ", "BCR ", "BDY ", "BEL ", "BEM ", "BEN ", "BGC ", "BGQ ", "BGR ", "BHI ",
    "BHO ", "BIK ", "BIL ", "BIS ", "BJJ ", "BKF ", "BLI ", "BLK ", "BLN ", "BLT ", "BMB ", "BML ",
    "BOS ", "BPY ", "BRE ", "BRH ", "BRI ", "BRM ", "BRX ", "BSH ", "BSK ", "BTD ", "BTI ", "BTK ",
    "BTM ", "BTS ", "BTX ", "BTZ ", "BUG ", "BYV ", "CAK ", "CAT ", "CBK ", "CCHN", "CEB ", "CGG ",
    "CHA ", "CHE ", "CHG ", "CHH ", "CHI ", "CHK ", "CHK0", "CHO ", "CHP ", "CHR ", "CHU ", "CHY ",
    "CJA ", "CJM ", "CMR ", "COP ", "COR ", "COS ", "CPP ", "CRE ", "CRR ", "CRT ", "CSB ", "CSL ",
    "CSY ", "CTG ", "CUK ", "DAG ", "DAN ", "DAR ", "DAX ", "DCR ", "DEU ", "DGO ", "DGR ", "DHG ",
    "DHV ", "DIQ ", "DIV ", "DJR ", "DJR0", "DNG ", "DNJ ", "DNK ", "DRI ", "DUJ ", "DUN ", "DZN ",
    "EBI ", "ECR ", "EDO ", "EFI ", "ELL ", "EMK ", "ENG ", "ERZ ", "ESP ", "ESU ", "ETI ", "EUQ ",
    "EVK ", "EVN ", "EWE ", "FAN ", "FAN0", "FAR ", "FAT ", "FIN ", "FJI ", "FLE ", "FMP ", "FNE ",
    "FON ", "FOS ", "FRA ", "FRC ", "FRI ", "FRL ", "FRP ", "FTA ", "FUL ", "FUV ", "GAD ", "GAE ",
    "GAG ", "GAL ", "GAR ", "GAW ", "GEZ ", "GIH ", "GIL ", "GIL0", "GKP ", "GLK ", "GMZ ", "GNN ",
    "GOG ", "GON ", "GRN ", "GRO ", "GUA ", "GUC ", "GUF ", "GUJ ", "GUZ ", "HAI ", "HAL ", "HAR ",
    "HAU ", "HAW ", "HAY ", "HAZ ", "HBN ", "HER ", "HIL ", "HIN ", "HMA ", "HMN ", "HMO ", "HND ",
    "HO  ", "HRI ", "HRV ", "HUN ", "HYE ", "HYE0", "IBA ", "IBB ", "IBO ", "IDO ", "IJO ", "ILE ",
    "ILO ", "INA ", "IND ", "ING ", "INU ", "IPK ", "IPPH", "IRI ", "IRT ", "ISL ", "ISM ", "ITA ",
    "IWR ", "JAM ", "JAN ", "JAV ", "JBO ", "JCT ", "JII ", "JUD ", "JUL ", "KAB ", "KAB0", "KAC ",
    "KAL ", "KAN ", "KAR ", "KAT ", "KAZ ", "KDE ", "KEA ", "KEB ", "KEK ", "KGE ", "KHA ", "KHK ",
    "KHM ", "KHS ", "KHT ", "KHV ", "KHW ", "KIK ", "KIR ", "KIS ", "KIU ", "KJD ", "KJP ", "KJZ ",
    "KKN ", "KLM ", "KMB ", "KMN ", "KMO ", "KMS ", "KMZ ", "KNR ", "KOD ", "KOH ", "KOK ", "KOM ",
    "KON ", "KON0", "KOP ", "KOR ", "KOS ", "KOZ ", "KPL ", "KRI ", "KRK ", "KRL ", "KRM ", "KRN ",
    "KRT ", "KSH ", "KSH0", "KSI ", "KSM ", "KSW ", "KUA ", "KUI ", "KUL ", "KUM ", "KUR ", "KUU ",
    "KUY ", "KYK ", "KYU ", "LAD ", "LAH ", "LAK ", "LAM ", "LAO ", "LAT ", "LAZ ", "LCR ", "LDK ",
    "LEZ ", "LIJ ", "LIM ", "LIN ", "LIS ", "LJP ", "LKI ", "LMA ", "LMB ", "LMO ", "LMW ", "LOM ",
    "LRC ", "LSB ", "LSM ", "LTH ", "LTZ ", "LUA ", "LUB ", "LUG ", "LUH ", "LUO ", "LVI ", "MAD ",
    "MAG ", "MAH ", "MAJ ", "MAK ", "MAL ", "MAM ", "MAN ", "MAP ", "MAR ", "MAW ", "MBN ", "MBO ",
    "MCH ", "MCR ", "MDE ", "MDR ", "MEN ", "MER ", "MFA ", "MFE ", "MIN ", "MIZ ", "MKD ", "MKR ",
    "MKW ", "MLE ", "MLG ", "MLN ", "MLR ", "MLY ", "MND ", "MNG ", "MNI ", "MNK ", "MNX ", "MOH ",
    "MOK ", "MOL ", "MON ", "MOR ", "MOS ", "MRI ", "MTH ", "MTS ", "MUN ", "MUS ", "MWL ", "MWW ",
    "MYN ", "MZN ", "NAG ", "NAH ", "NAN ", "NAP ", "NAS ", "NAU ", "NAV ", "NCR ", "NDB ", "NDC ",
    "NDG ", "NDS ", "NEP ", "NEW ", "NGA ", "NGR ", "NHC ", "NIS ", "NIU ", "NKL ", "NKO ", "NLD ",
    "NOE ", "NOG ", "NOR ", "NOV ", "NSM ", "NSO ", "NTA ", "NTO ", "NYM ", "NYN ", "NZA ", "OCI ",
    "OCR ", "OJB ", "ORI ", "ORO ", "OSS ", "PAA ", "PAG ", "PAL ", "PAM ", "PAN ", "PAP ", "PAP0",
    "PAS ", "PAU ", "PCC ", "PCD ", "PDC ", "PGR ", "PHK ", "PIH ", "PIL ", "PLG ", "PLK ", "PMS ",
    "PNB ", "POH ", "PON ", "PRO ", "PTG ", "PWO ", "QIN ", "QUC ", "QUH ", "QUZ ", "QVI ", "QWH ",
    "RAJ ", "RAR ", "RBU ", "RCR ", "REJ ", "RIA ", "RIF ", "RIT ", "RKW ", "RMS ", "RMY ", "ROM ",
    "ROY ", "RSY ", "RTM ", "RUA ", "RUN ", "RUP ", "RUS ", "SAD ", "SAN ", "SAS ", "SAT ", "SAY ",
    "SCN ", "SCO ", "SCS ", "SEK ", "SEL ", "SGA ", "SGO ", "SGS ", "SHI ", "SHN ", "SIB ", "SID ",
    "SIG ", "SKS ", "SKY ", "SLA ", "SLV ", "SML ", "SMO ", "SNA ", "SNA0", "SND ", "SNH ", "SNK ",
    "SOG ", "SOP ", "SOT ", "SQI ", "SRB ", "SRD ", "SRK ", "SRR ", "SSL ", "SSM ", "STQ ", "SUK ",
    "SUN ", "SUR ", "SVA ", "SVE ", "SWA ", "SWK ", "SWZ ", "SXT ", "SXU ", "SYL ", "SYR ", "SYRE",
    "SYRJ", "SYRN", "SZL ", "TAB ", "TAJ ", "TAM ", "TAT ", "TCR ", "TDD ", "TEL ", "TET ", "TGL ",
    "TGN ", "TGR ", "TGY ", "THA ", "THT ", "TIB ", "TIV ", "TKM ", "TMH ", "TMN ", "TNA ", "TNE ",
    "TNG ", "TOD ", "TOD0", "TPI ", "TRK ", "TSG ", "TSJ ", "TUA ", "TUL ", "TUM ", "TUV ", "TVL ",
    "TWI ", "TYZ ", "TZM ", "TZO ", "UDM ", "UKR ", "UMB ", "URD ", "USB ", "UYG ", "UZB ", "VEC ",
    "VEN ", "VIT ", "VOL ", "VRO ", "WA  ", "WAG ", "WAR ", "WCR ", "WEL ", "WLF ", "WLN ", "WTM ",
    "XBD ", "XHS ", "XJB ", "XKF ", "XOG ", "XPE ", "YAK ", "YAO ", "YAP ", "YBA ", "YCR ", "YIC ",
    "YIM ", "ZEA ", "ZGH ", "ZHA ", "ZHH ", "ZHP ", "ZHS ", "ZHT ", "ZHTM", "ZND ", "ZUL ", "ZZA ",
];

pub const VALID_FEATURE_TAGS: [&str; 122] = [
    "aalt", "abvf", "abvm", "abvs", "afrc", "akhn", "blwf", "blwm", "blws", "c2pc", "c2sc", "calt",
    "case", "ccmp", "cfar", "chws", "cjct", "clig", "cpct", "cpsp", "cswh", "curs", "dist", "dlig",
    "dnom", "dtls", "expt", "falt", "fin2", "fin3", "fina", "flac", "frac", "fwid", "half", "haln",
    "halt", "hist", "hkna", "hlig", "hngl", "hojo", "hwid", "init", "isol", "ital", "jalt", "jp04",
    "jp78", "jp83", "jp90", "kern", "lfbd", "liga", "ljmo", "lnum", "locl", "ltra", "ltrm", "mark",
    "med2", "medi", "mgrk", "mkmk", "mset", "nalt", "nlck", "nukt", "numr", "onum", "opbd", "ordn",
    "ornm", "palt", "pcap", "pkna", "pnum", "pref", "pres", "pstf", "psts", "pwid", "qwid", "rand",
    "rclt", "rkrf", "rlig", "rphf", "rtbd", "rtla", "rtlm", "ruby", "rvrn", "salt", "sinf", "size",
    "smcp", "smpl", "ssty", "stch", "subs", "sups", "swsh", "titl", "tjmo", "tnam", "tnum", "trad",
    "twid", "unic", "valt", "vatu", "vchw", "vert", "vhal", "vjmo", "vkna", "vkrn", "vpal", "vrt2",
    "vrtr", "zero",
];

pub const CJK_UNICODE_RANGES: [Range<u32>; 20] = [
    0x1100..0x11FF,   // Hangul Jamo
    0x3040..0x309F,   // Hiragana
    0x30A0..0x30FF,   // Katakana
    0x31F0..0x31FF,   // Katakana Phonetic Extensions
    0x3100..0x312F,   // Bopomofo
    0x31A0..0x31BF,   // Bopomofo Extended (Bopomofo)
    0x3130..0x318F,   // Hangul Compatibility Jamo
    0x3200..0x32FF,   // Enclosed CJK Letters and Months
    0x3300..0x33FF,   // CJK Compatibility
    0xAC00..0xD7AF,   // Hangul Syllables
    0x4E00..0x9FFF,   // CJK Unified Ideographs
    0x2E80..0x2EFF,   // CJK Radicals Supplement (CJK Unified Ideographs)
    0x2F00..0x2FDF,   // Kangxi Radicals (CJK Unified Ideographs)
    0x2FF0..0x2FFF,   // Ideographic Description Characters (CJK Unified Ideographs)
    0x3400..0x4DBF,   // CJK Unified Ideographs Extension A (CJK Unified Ideographs)
    0x20000..0x2A6DF, // CJK Unified Ideographs Extension B (CJK Unified Ideographs)
    0x3190..0x319F,   // Kanbun (CJK Unified Ideographs)
    0x31C0..0x31EF,   // CJK Strokes
    0xF900..0xFAFF,   // CJK Compatibility Ideographs (CJK Strokes)
    0x2F800..0x2FA1F, // CJK Compatibility Ideographs Supplement (CJK Strokes)
];
